use rusqlite::{Connection,Result};

pub fn get_lotdata(db_path: &str, lot_name: &str) -> Result<(Vec<String>, Vec<Vec<String>>), Box<dyn std::error::Error>> {
    let db = Connection::open(db_path)?;
    
    let mut stmt = db.prepare(
        "SELECT *
         FROM chipdata
         WHERE lot_name = ?",
    )?;
    
    let mut column_names: Vec<String> = stmt.column_names()
        .iter()
        .map(|&name| name.to_string())
        .collect();
    
    column_names.drain(..1);
    column_names.drain(column_names.len()-3..);
    
    let mut lot_unit_vec: Vec<Vec<String>> = Vec::new();
    
    let rows = stmt.query_map([lot_name], |row| {
        let column_count = row.as_ref().column_count();
        let mut row_data = Vec::new();
        
        // serialを先に取得（元のインデックス4）
        let serial_value: Result<String, _> = row.get(4);
        let (serial_num, flag) = match serial_value {
            Ok(v) => {
                match v.parse::<i32>() {
                    Ok(s) => (s, true),
                    Err(_) => (i32::MAX, false)
                }
            },
            Err(_) => (i32::MAX, false)
        };
        
        // 全カラムのデータを取得
        for i in 0..column_count {
            let value: Result<String, _> = row.get(i);
            match value {
                Ok(v) => row_data.push(v),
                Err(_) => row_data.push(String::new()),
            }
        }
        
        // row_dataの最初の要素と最後の3つの要素を取り除く
        row_data.drain(..1);
        row_data.drain(row_data.len()-3..);
        
        // flagに関わらず、serial番号とデータのペアを返す
        Ok((serial_num, flag, row_data))
    })?;

    // serial番号付きでデータを収集（flagがtrueのものだけ）
    let mut lot_unit_with_serial: Vec<(i32, Vec<String>)> = Vec::new();
    for row in rows {
        let (serial, flag, data) = row?;
        if flag {
            lot_unit_with_serial.push((serial, data));
        }
    }
    
    // serial番号で昇順にソート
    lot_unit_with_serial.sort_by_key(|(serial, _)| *serial);
    
    // データ部分だけを取り出す
    lot_unit_vec = lot_unit_with_serial.into_iter()
        .map(|(_, data)| data)
        .collect();
    
    Ok((column_names, lot_unit_vec))
}
