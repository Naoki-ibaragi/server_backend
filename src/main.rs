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

// グローバル設定
static DB_PATH: Lazy<String> = Lazy::new(|| {
    env::var("DB_PATH").unwrap_or("C:\\chiptest.db".to_string())
});

static DB_TABLE_JSON_PATH: Lazy<String> = Lazy::new(|| {
    env::var("DB_TABLE_JSON_PATH").unwrap_or("C:\\workspace\\server_backend\\assets\\dbtable.json".to_string())
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
    println!("{:?}",data);
    let lotdata=match get_lotdata(&DB_PATH, &data.lot_name){
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
    let lotdata=match get_alarmdata(&DB_PATH, &DB_TABLE_JSON_PATH,&data.machine_name,&ALARM_JSON_PATH){
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
    let s=match fs::read_to_string(&*DB_TABLE_JSON_PATH){
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
#[post("/get_graphdata")]
async fn get_graphdata(graph_condition: web::Json<GraphCondition>) -> HttpResponse {
    let mut grid_len_x=0.;
    let mut grid_len_y=0.;
    let mut grid_data_initial=GridData{x_min:0,y_min:0,grid_x:0.,grid_y:0.};
    println!("{:?}",graph_condition);

    let (success,message,graph_data,grid_data)=match get_graphdata_from_db(&DB_PATH, &graph_condition){
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
            .service(get_graphdata)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
