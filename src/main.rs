use actix_web::{post, web, App, HttpResponse, HttpServer};
use actix_cors::Cors;
use std::collections::HashMap;
use variants::{LotData,MachineData};
use graph::variants::GraphCondition;
use std::{env,fs};
use once_cell::sync::Lazy;
use sqlx::PgPool;
use tracing::{info, error, debug};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

use crate::graph::variants::{GridData};
use crate::lotdata::get_lotdata;
use crate::alarmdata::get_alarmdata;
use crate::graph::graphdata::get_graphdata_from_db;
use crate::operating_data::get_operating_data;

mod lotdata;
mod alarmdata;
mod variants;
mod graph;
mod operating_data;

static DB_URL: Lazy<String> = Lazy::new(|| {
    env::var("DB_URL").unwrap_or("postgresql://postgres:password@localhost:5432/chiptest".to_string())
});

static ALARM_JSON_PATH: Lazy<String> = Lazy::new(|| {
    env::var("ALARM_JSON_PATH").unwrap_or("C:\\workspace\\server_backend\\assets\\alarm.json".to_string())
});

// アプリケーション状態（DB接続プールを保持）
struct AppState {
    db_pool: PgPool,
} 

// ロット単位のデータを返す
//Input:lot_number
//Output:稼働データ
#[post("/download_lot")]
async fn download_lot(
    state: web::Data<AppState>,
    data: web::Json<LotData>
) -> HttpResponse {
    let success;
    let message;
    let lotdata=match get_lotdata(&state.db_pool,&data.lot_name).await{
        Ok(v)=>{
            success=true;
            message="success!".to_string();
            info!("Successfully retrieved lot data for lot_name: {}", data.lot_name);
            info!("{:?}",v[0]);
            info!("{:?}",v[1]);
            v
        },
        Err(e)=>{
            success=false;
            message=format!("{}",e);
            error!("Failed to retrieve lot data for lot_name: {}, error: {}", data.lot_name, e);
            vec![]
        }
    };

    let response = serde_json::json!({
        "success":success,
        "message":message,
        "lot_data": lotdata
    });

    HttpResponse::Ok().json(response)
}

#[post("/download_alarm")]
async fn download_alarm(
    state: web::Data<AppState>,
    data: web::Json<MachineData>
) -> HttpResponse {
    let success;
    let message;
    debug!("Received alarm data request: {:?}", data);
    debug!("ALARM_JSON_PATH: {}", &*ALARM_JSON_PATH);
    let lotdata=match get_alarmdata(&state.db_pool, &ALARM_JSON_PATH,data.machine_id,&data.start_date,&data.end_date).await{
        Ok(v)=>{
            success=true;
            message="success!".to_string();
            info!("Successfully retrieved alarm data for machine_id: {}", data.machine_id);
            v
        },
        Err(e)=>{
            success=false;
            message=format!("{}",e);
            error!("Failed to retrieve alarm data for machine_id: {}, error: {}", data.machine_id, e);
            HashMap::new()
        }
    };

    let response = serde_json::json!({
        "success":success,
        "message":message,
        "alarm_data": lotdata,
    });

    HttpResponse::Ok().json(response)
}

#[post("/get_machine_list")]
async fn get_machine_list()->HttpResponse{

    let response=serde_json::json!({
        "success":true,
        "message":"success",
        "machine_list":[1,2,3,4,5,6,7,8]
    });

    HttpResponse::Ok().json(response)

}

///グラフデータを返す
#[post("/get_graphdata")]
async fn get_graphdata(
    state: web::Data<AppState>,
    graph_condition: web::Json<GraphCondition>
) -> HttpResponse {
    let grid_data_initial=GridData{x_min:0,y_min:0,grid_x:0.,grid_y:0.,histogram_bin_info:None};
    debug!("Received graph data request: {:?}", graph_condition);

    let (success,message,graph_data,grid_data)=match get_graphdata_from_db(&state.db_pool, &graph_condition).await{
        Ok(data)=>{
            info!("Successfully retrieved graph data for graph_type: {}", graph_condition.graph_type);
            (true, "success".to_string(), data.0,data.1)
        },
        Err(e)=>{
            error!("Failed to retrieve graph data, error: {}", e);
            (false, format!("{}",e),HashMap::new(),grid_data_initial)
        }
    };

    let response=serde_json::json!({
        "success":success,
        "message":message,
        "graph_data":graph_data,
        "grid_data":grid_data,
    });

    HttpResponse::Ok().json(response)
}

#[post("/download_operating_data")]
async fn download_operating_data(
    state: web::Data<AppState>,
    data: web::Json<MachineData>
) -> HttpResponse {
    let success;
    let message;
    let summary_data=match get_operating_data(&state.db_pool,data.machine_id,&data.start_date,&data.end_date).await{
        Ok(v)=>{
            success=true;
            message="success!".to_string();
            info!("Successfully retrieved lot data for machine_id: {}", data.machine_id);
            info!("{:?}",v[0]);
            info!("{:?}",v[1]);
            v
        },
        Err(e)=>{
            success=false;
            message=format!("{}",e);
            error!("Failed to retrieve lot data for machine_id: {}, error: {}", data.machine_id, e);
            vec![]
        }
    };

    let response = serde_json::json!({
        "success":success,
        "message":message,
        "summary_data": summary_data
    });

    HttpResponse::Ok().json(response)
}



// --- メイン ---
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ログディレクトリのパスを設定（環境変数で上書き可能）
    let log_dir = env::var("LOG_DIR").unwrap_or_else(|_| "./logs".to_string());

    // ログディレクトリを作成
    fs::create_dir_all(&log_dir)?;

    // 日次ローテーションのファイルアペンダーを作成
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        &log_dir,
        "app.log"
    );

    // 環境変数からログレベルを設定（デフォルト: info）
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("debug"));

    // ログサブスクライバーを設定
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_writer(file_appender).with_ansi(false)) // ファイル出力
        .with(fmt::layer().with_writer(std::io::stdout)) // コンソール出力
        .init();

    info!("Logging initialized. Log directory: {}", log_dir);

    // DB接続プールを作成
    info!("Connecting to database: {}", &*DB_URL);
    let db_pool = PgPool::connect(&DB_URL)
        .await
        .expect("Failed to create database connection pool");

    info!("Database connection pool created successfully");
    info!("Starting HTTP server on 0.0.0.0:8080");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "OPTIONS"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::ACCEPT,
                actix_web::http::header::CONTENT_TYPE,
            ])
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(AppState {
                db_pool: db_pool.clone(),
            }))
            .wrap(cors)
            .service(download_lot)
            .service(download_alarm)
            .service(get_machine_list)
            .service(get_graphdata)
            .service(download_operating_data)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
