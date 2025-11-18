use actix_web::{post, web, App, HttpResponse, HttpServer};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use variants::{LotData,MachineData,GraphCondition};
use std::env;

use crate::lotdata::get_lotdata;
use crate::alarmdata::get_alarmdata;

mod lotdata;
mod alarmdata;
mod variants;

// ロット単位のデータを返す
//Input:lot_number
//Output:稼働データ
#[post("/download_lot")]
async fn download_lot(data: web::Json<LotData>) -> HttpResponse {
    let success;
    let message;
    println!("{:?}",data);
    let db_path:String=env::var("DB_PATH").unwrap_or("C:\\chiptest.db".to_string());
    let lotdata=match get_lotdata(&db_path, &data.lot_name){
        Ok(v)=>{
            success=true;
            message="success!".to_string();
            v
        },
        Err(e)=>{
            success=false;
            message=format!("{}",e);
            (vec![],vec![])
        }
    };

    let response = serde_json::json!({
        "success":success,
        "message":message,
        "lot_header": lotdata.0,
        "lot_data": lotdata.1
    });

    HttpResponse::Ok().json(response)
}

#[post("/download_alarm")]
async fn download_alarm(data: web::Json<MachineData>) -> HttpResponse {
    let success;
    let message;
    println!("{:?}",data);
    let db_path:String=env::var("DB_PATH").unwrap_or("C:\\chiptest.db".to_string());
    let dbtable_json_path:String=env::var("DB_TABLE_JSON_PATH").unwrap_or("C:\\workspace\\server_backend\\assets\\dbtable.json".to_string());
    let alarm_json_path:String=env::var("ALARM_JSON_PATH").unwrap_or("C:\\workspace\\server_backend\\assets\\alarm.json".to_string());
    let lotdata=match get_alarmdata(&db_path, &dbtable_json_path,&data.machine_name,&alarm_json_path){
        Ok(v)=>{
            success=true;
            message="success!".to_string();
            v
        },
        Err(e)=>{
            success=false;
            message=format!("{}",e);
            (vec![],vec![])
        }
    };

    let response = serde_json::json!({
        "success":success,
        "message":message,
        "lot_header": lotdata.0,
        "lot_data": lotdata.1
    });

    HttpResponse::Ok().json(response)
}

// --- メイン ---
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("0.0.0.0:5174")
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
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
