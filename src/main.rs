use actix_web::{post, web, App, HttpResponse, HttpServer};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use indexmap::IndexMap;
use variants::{LotData,MachineData,AlarmDetail};
use graph::variants::GraphCondition;
use std::{env,fs};
use once_cell::sync::Lazy;

use crate::graph::variants::{GridData};
use crate::lotdata::get_lotdata;
use crate::alarmdata::get_alarmdata;
use crate::graph::graphdata::get_graphdata_from_db;

mod lotdata;
mod alarmdata;
mod variants;
mod graph;

static DB_URL: Lazy<String> = Lazy::new(|| {
    env::var("DB_URL").unwrap_or("postgresql://postgres:password@localhost:5432/chiptest".to_string())
});

static ALARM_JSON_PATH: Lazy<String> = Lazy::new(|| {
    env::var("ALARM_JSON_PATH").unwrap_or("C:\\workspace\\server_backend\\assets\\alarm.json".to_string())
}); 

// ロット単位のデータを返す
//Input:lot_number
//Output:稼働データ
#[post("/download_lot")]
async fn download_lot(data: web::Json<LotData>) -> HttpResponse {
    let success;
    let message;
    let lotdata=match get_lotdata(&DB_URL,&data.lot_name).await{
        Ok(v)=>{
            success=true;
            message="success!".to_string();
            v
        },
        Err(e)=>{
            success=false;
            message=format!("{}",e);
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
async fn download_alarm(data: web::Json<MachineData>) -> HttpResponse {
    let success;
    let message;
    println!("{:?}",data);
    println!("ALARM_JSON_PATH: {}", &*ALARM_JSON_PATH);
    let lotdata=match get_alarmdata(&DB_URL, &ALARM_JSON_PATH,data.machine_id,&data.start_date,&data.end_date).await{
        Ok(v)=>{
            success=true;
            message="success!".to_string();
            v
        },
        Err(e)=>{
            success=false;
            message=format!("{}",e);
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
async fn get_graphdata(graph_condition: web::Json<GraphCondition>) -> HttpResponse {
    let grid_data_initial=GridData{x_min:0,y_min:0,grid_x:0.,grid_y:0.,histogram_bin_info:None};
    println!("{:?}",graph_condition);

    let (success,message,graph_data,grid_data)=match get_graphdata_from_db(&DB_URL, &graph_condition).await{
        Ok(data)=>{(true, "success".to_string(), data.0,data.1)},
        Err(e)=>{(false, format!("{}",e),HashMap::new(),grid_data_initial)}
    };

    let response=serde_json::json!({
        "success":success,
        "message":message,
        "graph_data":graph_data,
        "grid_data":grid_data,
    });

    HttpResponse::Ok().json(response)
}

// --- メイン ---
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
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
            .wrap(cors)
            .service(download_lot)
            .service(download_alarm)
            .service(get_machine_list)
            .service(get_graphdata)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
