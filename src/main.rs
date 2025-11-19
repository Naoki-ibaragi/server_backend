use actix_web::{post, web, App, HttpResponse, HttpServer};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use indexmap::IndexMap;
use variants::{LotData,MachineData,AlarmDetail};
use graph::variants::GraphCondition;
use std::{env,fs};

use crate::lotdata::get_lotdata;
use crate::alarmdata::get_alarmdata;
use crate::graph::graphdata::get_graphdata_from_db;

mod lotdata;
mod alarmdata;
mod variants;
mod graph;

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
            (
                HashMap::new(),
                AlarmDetail {
                    ld_alarm: HashMap::new(),
                    dc1_alarm: HashMap::new(),
                    ac1_alarm: HashMap::new(),
                    ac2_alarm: HashMap::new(),
                    dc2_alarm: HashMap::new(),
                    ip_alarm: HashMap::new(),
                    uld_alarm: HashMap::new(),
                }
            )
        }
    };

    let response = serde_json::json!({
        "success":success,
        "message":message,
        "alarm_data": lotdata.0,
        "alarm_header": lotdata.1
    });

    HttpResponse::Ok().json(response)
}

#[post("/get_machine_list")]
async fn get_machine_list()->HttpResponse{
    let dbtable_json_path:String=env::var("DB_TABLE_JSON_PATH").unwrap_or("C:\\workspace\\server_backend\\assets\\dbtable.json".to_string());
    let s=match fs::read_to_string(dbtable_json_path){
        Ok(s)=>s,
        Err(e)=>{
            "{}".to_string()
        }
    };

    let (success, message, machine_list) = match serde_json::from_str::<IndexMap<String, String>>(&s){
        Ok(table_map)=>{
            let mut machine_list:Vec<String>=vec![];
            for machine in table_map.keys(){
                machine_list.push(machine.clone());
            }
            (true, "success".to_string(), machine_list)
        }
        Err(e)=>{
            let machine_list:Vec<String>=vec![];
            (false, format!("{}",e), machine_list)
        }
    };

    let response=serde_json::json!({
        "success":success,
        "message":message,
        "machine_list":machine_list
    });

    HttpResponse::Ok().json(response)

}

///グラフデータを返す
#[post("/get_graph_data")]
async fn get_graph_data(graph_condition: web::Json<GraphCondition>) -> HttpResponse {




    let response=serde_json::json!({
    });

    HttpResponse::Ok().json(response)


}

// --- メイン ---
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("http://localhost:5174")
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
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
