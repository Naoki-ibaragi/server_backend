use sqlx::{PgPool, Row,Column};
use serde::Serialize;
use chrono;

#[derive(Debug, Serialize)]
pub enum DBData {
    Num(i32),
    Str(String),
    None
}

pub async fn get_lotdata(database_url:&str,lot_name: &str) -> Result<Vec<Vec<DBData>>, Box<dyn std::error::Error>> {
    // PostgreSQL接続
    let pool = PgPool::connect(database_url).await?;

    //最初にlotdateテーブルからロットのstart_timeとend_timeを取得
    let sql="SELECT start_date, end_date FROM lotdate WHERE lot_name = $1";
    let metadata = sqlx::query(sql).bind(lot_name)
    .fetch_one(&pool).await?;

    let start_date: chrono::NaiveDateTime = metadata.try_get("start_date")?;
    let end_date: chrono::NaiveDateTime = metadata.try_get("end_date")?;

    // PostgreSQLではパーティションテーブルCHIPDATAを直接クエリ可能
    // シリアル番号の昇順で並び替える
    let sql = "SELECT * FROM CHIPDATA WHERE lot_name = $1 AND ld_pickup_date BETWEEN $2 AND $3 ORDER BY serial ASC";

    println!("=== SQL ===");
    println!("{}", sql);
    println!("lot_name: {}", lot_name);
    println!("===========");

    let rows = sqlx::query(sql)
        .bind(lot_name).bind(start_date).bind(end_date)
        .fetch_all(&pool)
        .await?;

    let mut lot_unit_vec = Vec::new();

    for row in rows {
        let column_count = row.len();
        let mut row_data = Vec::new();

        // カラムインデックス1から開始(id列をスキップ)
        for i in 1..column_count {
            let column = &row.columns()[i];
            let column_name = column.name();

            // 型に応じてデータを取得
            let value = if let Ok(v) = row.try_get::<i32, _>(i) {
                DBData::Num(v)
            } else if let Ok(v) = row.try_get::<String, _>(i) {
                DBData::Str(v)
            } else if let Ok(v) = row.try_get::<Option<i32>, _>(i) {
                match v {
                    Some(n) => DBData::Num(n),
                    None => DBData::None,
                }
            } else if let Ok(v) = row.try_get::<Option<String>, _>(i) {
                match v {
                    Some(s) => DBData::Str(s),
                    None => DBData::None,
                }
            } else if let Ok(v) = row.try_get::<Option<chrono::NaiveDateTime>, _>(i) {
                match v {
                    Some(dt) => DBData::Str(dt.format("%Y-%m-%d %H:%M:%S").to_string()),
                    None => DBData::None,
                }
            } else {
                // その他の型はNoneとして扱う
                println!("Unknown type for column: {}", column_name);
                DBData::None
            };

            row_data.push(value);
        }

        lot_unit_vec.push(row_data);
    }

    pool.close().await;

    Ok(lot_unit_vec)
}
