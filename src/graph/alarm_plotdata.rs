/* プロット用のアラームデータを取得する関数 */
use sqlx::{PgPool, Row};
use std::error::Error;
use std::collections::HashMap;

use crate::graph::variants::*;

/* histogram */
//プロット分割しないヒストグラムのアラーム部分だけのデータを取得
pub async fn plot_histogram_without_unit_only_alarm_data(_total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,pool:&PgPool,sql:&str,params:&[String],bin_info:&HistogramBinInfo)->Result<(),Box<dyn Error>>{
    data_map.entry("alarm_data".to_string()).or_insert(vec![]);

    let mut query = sqlx::query(sql);
    for param in params {
        query = query.bind(param);
    }
    let rows_data = query.fetch_all(pool).await?;

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
pub async fn plot_histogram_with_unit_only_alarm_data(_total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,pool:&PgPool,sql:&str,params:&[String],bin_info:&HistogramBinInfo)->Result<(),Box<dyn Error>>{
    let mut query = sqlx::query(sql);
    for param in params {
        query = query.bind(param);
    }
    let rows_data = query.fetch_all(pool).await?;

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
