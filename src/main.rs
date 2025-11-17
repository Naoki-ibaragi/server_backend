use actix_web::{post, web, App, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};
use variants::{LotData,MachineData,GraphCondition};

use crate::lotdata::get_lotdata;

mod lotdata;
mod variants;

static DB_PATH:&str="D:\\chiptest.db";

// --- JSON 入力データ ---
#[derive(Deserialize)]
struct InputData {
    name: String,
    age: u32,
}

// --- JSON 出力データ ---
#[derive(Serialize)]
struct OutputData {
    message: String,
    is_adult: bool,
}

// ロット単位のデータを返す
//Input:lot_number
//Output:稼働データ
#[post("/download_lot")]
async fn download_lot(data: web::Json<LotData>) -> HttpResponse {
    let lotdata=match get_lotdata(DB_PATH, &data.lot_name){
        Ok(v)=>v,
        Err(e)=>(vec![],vec![])
    };

    let response = serde_json::json!({
        "lot_header": lotdata.0,
        "lot_data": lotdata.1
    });

    HttpResponse::Ok().json(response)
}

// --- メイン ---
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(download_lot)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
