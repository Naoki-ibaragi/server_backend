/* プロット用のアラームデータを取得する関数 */
use rusqlite::{Statement};
use std::error::Error;
use std::collections::HashMap;

use crate::graph::variants::*;

/* scatter */
//プロット分割しない散布図のアラーム部分だけのデータを取得
pub fn plot_scatterplot_without_unit_only_alarm_data(total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,stmt:&mut Statement)->Result<(),Box<dyn Error>>{
    data_map.entry("alarm_data".to_string()).or_insert(vec![]);

    let query_rows: Vec<Vec<i32>> = stmt.query_map([], |row| {
        let x_value: String = row.get(0)?;
        let y_value: String = row.get(1)?;
        Ok((x_value, y_value))
    })?
    .filter_map(|r| {
        let (x_val, y_val) = r.ok()?;
        let x = x_val.parse::<i32>().ok()?;
        let y = y_val.parse::<i32>().ok()?;
        Some(vec![x, y])
    })
    .collect();

    // 最初に全ての行をカウント（オプション：パフォーマンスが心配な場合は別途COUNT(*)で取得）
    // 以下のコードでは処理しながら報告していく方式を使用
    let rows= data_map.get_mut("alarm_data").unwrap();
    for (index,record) in query_rows.into_iter().enumerate(){
        rows.push(PlotData::Number(NumberData::new(record[0],record[1])));

        // 1000行ごとに進捗を報告
    }

    Ok(())
}

//プロット分割する散布図のデータを取得
pub fn plot_scatterplot_with_unit_only_alarm_data(total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,stmt:&mut Statement)->Result<(),Box<dyn Error>>{
    let query_rows: Vec<TmpData> = stmt.query_map([], |row| {
        let unit_name: String=row.get(0)?;
        let x_value: String = row.get(1)?;
        let y_value: String = row.get(2)?;
        Ok((unit_name,x_value, y_value))
    })?
    .filter_map(|r| {
        let (unit_name,x_val, y_val) = r.ok()?;
        let x = x_val.parse::<i32>().ok()?;
        let y = y_val.parse::<i32>().ok()?;
        let plot_data=PlotData::Number(NumberData::new(x,y));
        Some(TmpData::new(unit_name,plot_data))
    })
    .collect();

    for (index,record) in query_rows.into_iter().enumerate(){
        //unitがHashMapになければ追加
        let key_name=record.unit.clone()+"_alarm";
        if data_map.contains_key(&key_name){
            let rows=data_map.get_mut(&record.unit).unwrap();
            rows.push(match record.data{
                PlotData::Number(num_data)=>PlotData::Number(NumberData::new(num_data.x,num_data.y)),
                PlotData::Calendar(calender_data)=>PlotData::Calendar(CalenderData::new(calender_data.x,calender_data.y)),
                _=>return Err(Box::from(format!("不明なPlotData型が検出されました")))
            });
        }else{
            data_map.entry(key_name).or_insert(
        match record.data{
                    PlotData::Number(num_data)=>vec![PlotData::Number(NumberData::new(num_data.x,num_data.y))],
                    PlotData::Calendar(calender_data)=>vec![PlotData::Calendar(CalenderData::new(calender_data.x,calender_data.y))],
                    _=>return Err(Box::from(format!("不明なPlotData型が検出されました")))
                }
            );
        }

        // 1000行ごとに進捗を報告
    }

    Ok(())

}


/* histogram */
//プロット分割しないヒストグラムのアラーム部分だけのデータを取得
pub fn plot_histogram_without_unit_only_alarm_data(total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,stmt:&mut Statement)->Result<(),Box<dyn Error>>{

    //dataキーに値を入れる
    data_map.entry("alarm_data".to_string()).or_insert(vec![]);

    let query_rows: Vec<Vec<i32>> = stmt.query_map([], |row| {
        let x_value: String = row.get(0)?;
        Ok(x_value)
    })?
    .filter_map(|r| {
        let x_val = r.ok()?;
        let x = x_val.parse::<i32>().ok()?;
        Some(vec![x])
    })
    .collect();

    // 最初に全ての行をカウント（オプション：パフォーマンスが心配な場合は別途COUNT(*)で取得）
    // 以下のコードでは処理しながら報告していく方式を使用
    let rows= data_map.get_mut("alarm_data").unwrap();
    for (index,record) in query_rows.into_iter().enumerate(){
        rows.push(PlotData::Number1D(NumberData_1D::new(record[0])));

        // 1000行ごとに進捗を報告
    }

    Ok(())
}

//プロット分割するヒストグラムのアラームデータを取得
pub fn plot_histogram_with_unit_only_alarm_data(total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,stmt:&mut Statement)->Result<(),Box<dyn Error>>{
    let query_rows: Vec<TmpData_1D> = stmt.query_map([], |row| {
        let unit_name: String=row.get(0)?;
        let x_value: String = row.get(1)?;
        Ok((unit_name,x_value))
    })?
    .filter_map(|r| {
        let (unit_name,x_val) = r.ok()?;
        let x = x_val.parse::<i32>().ok()?;
        let plot_data=PlotData::Number1D(NumberData_1D::new(x));
        Some(TmpData_1D::new(unit_name,plot_data))
    })
    .collect();

    for (index,record) in query_rows.into_iter().enumerate(){
        //unitがHashMapになければ追加
        let key_name=record.unit.clone()+"_alarm";
        if data_map.contains_key(&key_name){
            let rows=data_map.get_mut(&key_name).unwrap();
            rows.push(match record.data{
                PlotData::Number1D(num_data)=>PlotData::Number1D(NumberData_1D::new(num_data.x)),
                _=>return Err(Box::from(format!("不明なPlotData型が検出されました")))
            });
        }else{
            data_map.entry(key_name).or_insert(
        match record.data{
                    PlotData::Number1D(num_data)=>vec![PlotData::Number1D(NumberData_1D::new(num_data.x))],
                    _=>return Err(Box::from(format!("不明なPlotData型が検出されました")))
                }
            );
        }

        // 1000行ごとに進捗を報告
    }

    Ok(())
}

/* Line */
//プロット分割しない散布図のアラーム部分だけのデータを取得
pub fn plot_lineplot_without_unit_only_alarm_data(total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,stmt:&mut Statement,graph_condition:&GraphCondition)->Result<(),Box<dyn Error>>{
    data_map.entry("alarm_data".to_string()).or_insert(vec![]);

    let query_rows:Vec<PlotData>=stmt.query_map([], |row| {
        let x_value: String = row.get(0)?;
        let y_value: String = row.get(1)?;
        Ok((x_value, y_value))
    })?
    .filter_map(|r| {
        if graph_condition.graph_x_item.contains("TIME"){
            let (x_val, y_val) = r.ok()?;
            if x_val.is_empty(){
                return None;
            }
            let y = y_val.parse::<i32>().ok()?;
            Some(PlotData::Calendar(CalenderData::new(x_val,y)))
        }else{
            let (x_val, y_val) = r.ok()?;
            let x = x_val.parse::<i32>().ok()?;
            let y = y_val.parse::<i32>().ok()?;
            Some(PlotData::Number(NumberData::new(x, y)))
        }
    })
    .collect();

    //HashMapのvecに書き込む
    let rows= data_map.get_mut("alarm_data").unwrap();
    for (index,record) in query_rows.into_iter().enumerate(){
        rows.push(match record{
            PlotData::Number(num_data)=>PlotData::Number(NumberData::new(num_data.x,num_data.y)),
            PlotData::Calendar(calender_data)=>PlotData::Calendar(CalenderData::new(calender_data.x,calender_data.y)),
            _=>return Err(Box::from(format!("不明なPlotData型が検出されました")))
        });

        // 1000行ごとに進捗を報告
    }

    Ok(())
}

//プロット分割する折れ線グラフのアラームデータを取得
pub fn plot_lineplot_with_unit_only_alarm_data(total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,stmt:&mut Statement,graph_condition:&GraphCondition)->Result<(),Box<dyn Error>>{
    let query_rows: Vec<TmpData> = stmt.query_map([], |row| {
        let unit_name: String=row.get(0)?;
        let x_value: String = row.get(1)?;
        let y_value: String = row.get(2)?;
        Ok((unit_name,x_value, y_value))
    })?
    .filter_map(|r| {
        let (unit_name,x_val, y_val) = r.ok()?;

        if graph_condition.graph_x_item.contains("TIME"){
            if x_val.is_empty(){
                return None;
            }
            let y = y_val.parse::<i32>().ok()?;
            let plot_data=PlotData::Calendar(CalenderData::new(x_val,y));
            Some(TmpData::new(unit_name,plot_data))
        }else{
            let x = x_val.parse::<i32>().ok()?;
            let y = y_val.parse::<i32>().ok()?;
            let plot_data=PlotData::Number(NumberData::new(x,y));
            Some(TmpData::new(unit_name,plot_data))
        }
    })
    .collect();

    for (index,record) in query_rows.into_iter().enumerate(){
        //unitがHashMapになければ追加
        let key_name=record.unit.clone()+"_alarm";
        if data_map.contains_key(&key_name){
            let rows=data_map.get_mut(&key_name).unwrap();
            rows.push(match record.data{
                PlotData::Number(num_data)=>PlotData::Number(NumberData::new(num_data.x,num_data.y)),
                PlotData::Calendar(calender_data)=>PlotData::Calendar(CalenderData::new(calender_data.x,calender_data.y)),
                _=>return Err(Box::from(format!("不明なPlotData型が検出されました")))
            });
        }else{
            data_map.entry(key_name).or_insert(
        match record.data{
                    PlotData::Number(num_data)=>vec![PlotData::Number(NumberData::new(num_data.x,num_data.y))],
                    PlotData::Calendar(calender_data)=>vec![PlotData::Calendar(CalenderData::new(calender_data.x,calender_data.y))],
                    _=>return Err(Box::from(format!("不明なPlotData型が検出されました")))
                }
            );
        }

        // 1000行ごとに進捗を報告
    }

    Ok(())
}

