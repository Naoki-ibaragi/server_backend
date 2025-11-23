use rusqlite::{Connection,Result,types::{ValueRef,Type}};
use serde::Serialize;

#[derive(Debug,Serialize)]
pub enum DBData{
    Num(i32),
    Str(String),
    None
}

pub fn get_lotdata(db_path: &str, lot_name: &str) -> Result<Vec<Vec<DBData>>, Box<dyn std::error::Error>> {
    let db = Connection::open(db_path)?;

    //テーブル一覧を取得
    let table_list_sql = "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'";
    let mut table_stmt = db.prepare(table_list_sql)?;
    let table_names: Vec<String> = table_stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<String>, _>>()?;

    println!("取得したテーブル一覧: {:?}", table_names);


    //各テーブルからデータを取得するためのUNION ALL SQLを生成
    let mut union_sql_parts = Vec::new();
    for table_name in &table_names {
        let sql = format!("SELECT * FROM {} WHERE lot_name='{}'", table_name,lot_name);
        union_sql_parts.push(sql);
    }

    //全テーブルのデータを統合
    //シリアル番号の昇順で並び替える
    let mut sql = union_sql_parts.join(" UNION ALL ");
    sql += " ORDER BY SERIAL ASC";

    println!("=== UNION ALL SQL ===");
    println!("{}", sql);
    println!("====================");

    let mut stmt=db.prepare(&sql)?;

    let rows = stmt.query_map([], |row| {
        let column_count = row.as_ref().column_count();
        let mut row_data = Vec::new();

        for i in 1..column_count {
            let v = row.get_ref(i)?;

            let value = match v.data_type() {
                Type::Integer => {
                    let n: i64 = v.as_i64()?;     // INTEGER は i64
                    DBData::Num(n as i32)        // i32 に落とすならキャスト
                }
                Type::Text => {
                    let s = v.as_str()?.to_string();
                    DBData::Str(s)
                }
                _ => DBData::None,
            };

            row_data.push(value);
        }

        Ok(row_data)
    })?;

    let mut lot_unit_vec = Vec::new();
    for r in rows {
        lot_unit_vec.push(r?);
    }

    Ok(lot_unit_vec)
}
