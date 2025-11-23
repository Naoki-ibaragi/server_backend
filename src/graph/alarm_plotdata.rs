/* プロット用のアラームデータを取得する関数 */
use rusqlite::{Statement};
use std::error::Error;
use std::collections::HashMap;

use crate::graph::variants::*;

/* scatter */
//プロット分割しない散布図のアラーム部分だけのデータを取得
pub fn plot_scatterplot_without_unit_only_alarm_data(total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,stmt:&mut Statement,graph_condition:&GraphCondition)->Result<(),Box<dyn Error>>{
    data_map.entry("alarm_data".to_string()).or_insert(vec![]);

    let query_rows: Vec<(XdimData,i32)> = stmt.query_map([], |row| {
        let x_value: XdimData = if graph_condition.graph_x_item.contains("DATE"){
            XdimData::StringData(row.get(0)?)
        }else{
            XdimData::NumberData(row.get(0)?)
        };
        let y_value: i32 = row.get(1)?;
        Ok((x_value,y_value))
    })?
    .filter_map(|r| r.ok())
    .collect();

    println!("query_rows collected: {} rows", query_rows.len());

    // 最初に全ての行をカウント（オプション：パフォーマンスが心配な場合は別途COUNT(*)で取得）
    // 以下のコードでは処理しながら報告していく方式を使用
    let rows= data_map.get_mut("alarm_data").unwrap();
    for (_index,record) in query_rows.into_iter().enumerate(){
        rows.push(PlotData::Scatter(ScatterPlotData{x_data:record.0,y_data:record.1}));
    }

    println!("Final data_map size: {}", rows.len());

    Ok(())

}

//プロット分割する散布図のデータを取得
pub fn plot_scatterplot_with_unit_only_alarm_data(total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,stmt:&mut Statement,graph_condition:&GraphCondition)->Result<(),Box<dyn Error>>{
    let query_rows: Vec<(String,XdimData,i32)> = stmt.query_map([], |row| {
        let unit_name: String=row.get(0)?;
        let x_value: XdimData = if graph_condition.graph_x_item.contains("DATE"){
            XdimData::StringData(row.get(1)?)
        }else{
            XdimData::NumberData(row.get(1)?)
        };
        let y_value: i32 = row.get(2)?;
        Ok((unit_name,x_value, y_value))
    })?
    .filter_map(|r| r.ok())
    .collect();

    for (index, record) in query_rows.into_iter().enumerate(){
        data_map.entry("alarm_".to_string()+&record.0).or_insert(vec![]).push(
            PlotData::Scatter(ScatterPlotData{x_data:record.1, y_data:record.2})
        );
    }
    Ok(())
}


/* histogram */
//プロット分割しないヒストグラムのアラーム部分だけのデータを取得
pub fn plot_histogram_without_unit_only_alarm_data(_total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,stmt:&mut Statement,bin_info:&HistogramBinInfo)->Result<(),Box<dyn Error>>{
    data_map.entry("alarm_data".to_string()).or_insert(vec![]);

    let query_rows: Vec<i32> = stmt.query_map([], |row| {
        let x_value: i32 = row.get(0)?;
        Ok(x_value)
    })?
    .filter_map(|r| r.ok())
    .collect();

    if query_rows.is_empty() || bin_info.bin_edges.is_empty(){
        return Ok(());
    }

    // 通常データと同じビンを使ってアラームデータを集計
    let x_min = bin_info.bin_edges[0];
    let bin_width = bin_info.bin_width;
    let bin_count = bin_info.bin_edges.len() - 1;

    let mut bin_counts = vec![0; bin_count];
    for value in query_rows {
        let bin_index = if bin_width == 0.0 {
            0
        } else {
            (((value - x_min) as f64 / bin_width) as usize).min(bin_count - 1)
        };
        bin_counts[bin_index] += 1;
    }

    // アラームデータを "alarm_data" キーに保存
    let rows = data_map.get_mut("alarm_data").unwrap();
    for (bin_index, count) in bin_counts.into_iter().enumerate() {
        rows.push(PlotData::BinnedHistogram(BinnedHistogramData {
            bin_index,
            count,
        }));
    }

    Ok(())

}

//プロット分割するヒストグラムのアラームデータを取得
pub fn plot_histogram_with_unit_only_alarm_data(_total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,stmt:&mut Statement,bin_info:&HistogramBinInfo)->Result<(),Box<dyn Error>>{
    let query_rows: Vec<(String,i32)> = stmt.query_map([], |row| {
        let unit_name: String=row.get(0)?;
        let x_value: i32 = row.get(1)?;
        Ok((unit_name,x_value))
    })?
    .filter_map(|r| r.ok())
    .collect();

    if query_rows.is_empty() || bin_info.bin_edges.is_empty(){
        return Ok(());
    }

    // 通常データと同じビンを使用
    let x_min = bin_info.bin_edges[0];
    let bin_width = bin_info.bin_width;
    let bin_count = bin_info.bin_edges.len() - 1;

    // ユニットごとにデータを分けてビン化
    let mut unit_data: HashMap<String, Vec<i32>> = HashMap::new();
    for (unit_name, value) in query_rows {
        unit_data.entry(unit_name).or_insert(vec![]).push(value);
    }

    // 各ユニットのアラームデータをビン化
    for (unit_name, values) in unit_data {
        let mut bin_counts = vec![0; bin_count];
        for value in values {
            let bin_index = if bin_width == 0.0 {
                0
            } else {
                (((value - x_min) as f64 / bin_width) as usize).min(bin_count - 1)
            };
            bin_counts[bin_index] += 1;
        }

        // BinnedHistogramDataとして格納
        for (bin_index, count) in bin_counts.into_iter().enumerate() {
            data_map.entry("alarm_".to_string()+&unit_name).or_insert(vec![]).push(
                PlotData::BinnedHistogram(BinnedHistogramData {
                    bin_index,
                    count,
                })
            );
        }
    }

    Ok(())

}

