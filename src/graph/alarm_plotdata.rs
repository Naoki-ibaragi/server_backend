/* プロット用のアラームデータを取得する関数 */
use sqlx::{PgPool, Row};
use std::error::Error;
use std::collections::HashMap;

use crate::graph::variants::*;

/* scatter */
//プロット分割しない散布図のアラーム部分だけのデータを取得
pub async fn plot_scatterplot_without_unit_only_alarm_data(total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,pool:&PgPool,sql:&str,graph_condition:&GraphCondition)->Result<(),Box<dyn Error>>{
    data_map.entry("alarm_data".to_string()).or_insert(vec![]);

    let rows_data = sqlx::query(sql)
        .fetch_all(pool)
        .await?;

    println!("query_rows collected: {} rows", rows_data.len());

    let rows = data_map.get_mut("alarm_data").unwrap();
    for row in rows_data {
        let y_opt: Option<i32> = row.try_get(1).ok().flatten();

        let x_is_valid = if graph_condition.graph_x_item.contains("DATE"){
            row.try_get::<Option<chrono::NaiveDateTime>, _>(0).ok().flatten().is_some()
        }else{
            row.try_get::<Option<i32>, _>(0).ok().flatten().is_some()
        };

        // XとYの両方がSomeの場合のみプッシュ
        if x_is_valid && y_opt.is_some() {
            let x_value: XdimData = if graph_condition.graph_x_item.contains("DATE"){
                XdimData::DateData(row.try_get(0).ok())
            }else{
                XdimData::NumberData(row.try_get(0).ok())
            };
            rows.push(PlotData::Scatter(ScatterPlotData{x_data:x_value,y_data:y_opt}));
        }
    }

    println!("Final data_map size: {}", rows.len());

    Ok(())

}

//プロット分割する散布図のデータを取得
pub async fn plot_scatterplot_with_unit_only_alarm_data(total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,pool:&PgPool,sql:&str,graph_condition:&GraphCondition)->Result<(),Box<dyn Error>>{
    let rows_data = sqlx::query(sql)
        .fetch_all(pool)
        .await?;

    for row in rows_data {
        let unit_name: String = row.try_get(0)?;
        let y_opt: Option<i32> = row.try_get(2).ok().flatten();

        let x_is_valid = if graph_condition.graph_x_item.contains("DATE"){
            row.try_get::<Option<chrono::NaiveDateTime>, _>(1).ok().flatten().is_some()
        }else{
            row.try_get::<Option<i32>, _>(1).ok().flatten().is_some()
        };

        // XとYの両方がSomeの場合のみプッシュ
        if x_is_valid && y_opt.is_some() {
            let x_value: XdimData = if graph_condition.graph_x_item.contains("DATE"){
                XdimData::DateData(row.try_get(1).ok())
            }else{
                XdimData::NumberData(row.try_get(1).ok())
            };
            data_map.entry("alarm_".to_string()+&unit_name).or_insert(vec![]).push(
                PlotData::Scatter(ScatterPlotData{x_data:x_value, y_data:y_opt})
            );
        }
    }
    Ok(())
}


/* histogram */
//プロット分割しないヒストグラムのアラーム部分だけのデータを取得
pub async fn plot_histogram_without_unit_only_alarm_data(_total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,pool:&PgPool,sql:&str,bin_info:&HistogramBinInfo)->Result<(),Box<dyn Error>>{
    data_map.entry("alarm_data".to_string()).or_insert(vec![]);

    let rows_data = sqlx::query(sql)
        .fetch_all(pool)
        .await?;

    let mut query_rows: Vec<i32> = Vec::new();
    for row in rows_data {
        if let Ok(Some(x_value)) = row.try_get::<Option<i32>, _>(0) {
            query_rows.push(x_value);
        }
    }

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
pub async fn plot_histogram_with_unit_only_alarm_data(_total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,pool:&PgPool,sql:&str,bin_info:&HistogramBinInfo)->Result<(),Box<dyn Error>>{
    let rows_data = sqlx::query(sql)
        .fetch_all(pool)
        .await?;

    let mut query_rows: Vec<(String,i32)> = Vec::new();
    for row in rows_data {
        let unit_name: String = row.try_get(0)?;
        if let Ok(Some(x_value)) = row.try_get::<Option<i32>, _>(1) {
            query_rows.push((unit_name, x_value));
        }
    }

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
