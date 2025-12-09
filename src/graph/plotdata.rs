use sqlx::{PgPool, Row};
use std::error::Error;
use std::collections::HashMap;

use crate::graph::variants::*;

//プロット分割しない散布図のデータを取得
pub async fn plot_scatterplot_without_unit(total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,pool:&PgPool,sql:&str,graph_condition:&GraphCondition)->Result<(),Box<dyn Error>>{
    data_map.entry("data".to_string()).or_insert(vec![]);

    let rows_data = sqlx::query(sql)
        .fetch_all(pool)
        .await?;

    println!("query_rows collected: {} rows", rows_data.len());

    let rows = data_map.get_mut("data").unwrap();
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
pub async fn plot_scatterplot_with_unit(total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,pool:&PgPool,sql:&str,graph_condition:&GraphCondition)->Result<(),Box<dyn Error>>{
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
            data_map.entry(unit_name).or_insert(vec![]).push(
                PlotData::Scatter(ScatterPlotData{x_data:x_value, y_data:y_opt})
            );
        }
    }
    Ok(())
}

//プロット分割しない折れ線グラフ(時系列プロット)のデータを取得
//LD_PICKUP_DATEでORDERされた状態でデータ取得済
pub async fn plot_lineplot_without_unit(total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,pool:&PgPool,sql:&str,graph_condition:&GraphCondition)->Result<(),Box<dyn Error>>{
    data_map.entry("data".to_string()).or_insert(vec![]);

    let rows_data = sqlx::query(sql)
        .fetch_all(pool)
        .await?;

    println!("query_rows collected: {} rows", rows_data.len());

    let rows = data_map.get_mut("data").unwrap();

    if graph_condition.alarm.codes.is_empty(){ //アラーム情報を取得しない場合
        for row in rows_data {
            let y_value: Option<i32> = row.try_get(1).ok().flatten();
            // Yがnullでない場合のみプッシュ
            if y_value.is_some() {
                rows.push(PlotData::Line(LinePlotData{y_data:y_value,is_alarm:false}));
            }
        }
    }else{
        let target_alarm_code:Vec<i32>=graph_condition.alarm.codes.clone(); //集計対象のアラームコードリスト
        for row in rows_data {
            let y_value: Option<i32> = row.try_get(1).ok().flatten();
            // Yがnullでない場合のみプッシュ
            if y_value.is_some() {
                let alarm_value: Option<i32> = row.try_get(2).ok().flatten();
                let is_alarm = alarm_value.map(|v| target_alarm_code.contains(&v)).unwrap_or(false);
                rows.push(PlotData::Line(LinePlotData{y_data:y_value,is_alarm}));
            }
        }
    }

    println!("Final data_map size: {}", rows.len());

    Ok(())
}

//プロット分割する折れ線グラフのデータを取得
pub async fn plot_lineplot_with_unit(total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,pool:&PgPool,sql:&str,graph_condition:&GraphCondition)->Result<(),Box<dyn Error>>{
    let rows_data = sqlx::query(sql)
        .fetch_all(pool)
        .await?;

    if graph_condition.alarm.codes.is_empty(){ //アラーム情報を取得しない場合
        for row in rows_data {
            let unit: String = row.try_get(0)?;
            let y_value: Option<i32> = row.try_get(2).ok().flatten();
            // Yがnullでない場合のみプッシュ
            if y_value.is_some() {
                data_map.entry(unit).or_insert(vec![]).push(
                    PlotData::Line(LinePlotData{y_data:y_value,is_alarm:false})
                );
            }
        }
    }else{
        let target_alarm_code:Vec<i32>=graph_condition.alarm.codes.clone(); //集計対象のアラームコードリスト
        for row in rows_data {
            let unit: String = row.try_get(0)?;
            let y_value: Option<i32> = row.try_get(2).ok().flatten();
            // Yがnullでない場合のみプッシュ
            if y_value.is_some() {
                let alarm_value: Option<i32> = row.try_get(3).ok().flatten();
                let is_alarm = alarm_value.map(|v| target_alarm_code.contains(&v)).unwrap_or(false);
                data_map.entry(unit).or_insert(vec![]).push(
                    PlotData::Line(LinePlotData{y_data:y_value,is_alarm})
                );
            }
        }
    }
    Ok(())
}

/* Heatmap(Histogram) */
//プロット分割しないヒストグラムのデータを取得
pub async fn plot_histogram_without_unit(_total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,pool:&PgPool,sql:&str,graph_condition:&GraphCondition)->Result<HistogramBinInfo,Box<dyn Error>>{
    //dataキーに値を入れる
    data_map.entry("data".to_string()).or_insert(vec![]);

    let rows_data = sqlx::query(sql)
        .fetch_all(pool)
        .await?;

    let mut query_rows: Vec<i32> = Vec::new();
    for row in rows_data {
        if let Ok(Some(x_value)) = row.try_get::<Option<i32>, _>(0) {
            query_rows.push(x_value);
        }
    }

    if query_rows.is_empty(){
        return Ok(HistogramBinInfo{
            bin_edges: vec![],
            bin_width: 0.0,
        });
    }

    // データの最小値と最大値を取得
    let x_min = *query_rows.iter().min().unwrap();
    let x_max = *query_rows.iter().max().unwrap();

    // ビン幅を計算
    let bin_width = (x_max - x_min) as f64 / graph_condition.bin_number as f64;

    // ビンの境界値を計算
    let mut bin_edges = Vec::new();
    for i in 0..=graph_condition.bin_number {
        bin_edges.push(x_min + (bin_width * i as f64) as i32);
    }

    // ビンごとの個数を集計
    let mut bin_counts = vec![0; graph_condition.bin_number as usize];
    for value in query_rows {
        let bin_index = if bin_width == 0.0 {
            0
        } else {
            (((value - x_min) as f64 / bin_width) as usize).min(graph_condition.bin_number as usize - 1)
        };
        bin_counts[bin_index] += 1;
    }

    // BinnedHistogramDataとして格納
    let rows = data_map.get_mut("data").unwrap();
    for (bin_index, count) in bin_counts.into_iter().enumerate() {
        rows.push(PlotData::BinnedHistogram(BinnedHistogramData {
            bin_index,
            count,
        }));
    }

    Ok(HistogramBinInfo {
        bin_edges,
        bin_width,
    })
}

//プロット分割するヒストグラムのデータを取得
pub async fn plot_histogram_with_unit(_total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,pool:&PgPool,sql:&str,graph_condition:&GraphCondition)->Result<HistogramBinInfo,Box<dyn Error>>{
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

    if query_rows.is_empty(){
        return Ok(HistogramBinInfo{
            bin_edges: vec![],
            bin_width: 0.0,
        });
    }

    // 全データの最小値と最大値を取得(全ユニットで同じビンを使用)
    let x_min = *query_rows.iter().map(|(_, v)| v).min().unwrap();
    let x_max = *query_rows.iter().map(|(_, v)| v).max().unwrap();

    // ビン幅を計算
    let bin_width = (x_max - x_min) as f64 / graph_condition.bin_number as f64;

    // ビンの境界値を計算
    let mut bin_edges = Vec::new();
    for i in 0..=graph_condition.bin_number {
        bin_edges.push(x_min + (bin_width * i as f64) as i32);
    }

    // ユニットごとにデータを分けてビン化
    let mut unit_data: HashMap<String, Vec<i32>> = HashMap::new();
    for (unit_name, value) in query_rows {
        unit_data.entry(unit_name).or_insert(vec![]).push(value);
    }

    // 各ユニットのデータをビン化
    for (unit_name, values) in unit_data {
        let mut bin_counts = vec![0; graph_condition.bin_number as usize];
        for value in values {
            let bin_index = if bin_width == 0.0 {
                0
            } else {
                (((value - x_min) as f64 / bin_width) as usize).min(graph_condition.bin_number as usize - 1)
            };
            bin_counts[bin_index] += 1;
        }

        // BinnedHistogramDataとして格納
        for (bin_index, count) in bin_counts.into_iter().enumerate() {
            data_map.entry(unit_name.clone()).or_insert(vec![]).push(
                PlotData::BinnedHistogram(BinnedHistogramData {
                    bin_index,
                    count,
                })
            );
        }
    }

    Ok(HistogramBinInfo {
        bin_edges,
        bin_width,
    })
}

/* Heatmap(DensityPlot) */
//プロット分割しない密度プロットのデータを取得
pub async fn plot_densityplot_without_unit(total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,pool:&PgPool,sql:&str,graph_condition:&GraphCondition)->Result<GridData,Box<dyn Error>>{
    data_map.entry("data".to_string()).or_insert(vec![]);

    let rows_data = sqlx::query(sql)
        .fetch_all(pool)
        .await?;

    //格子幅を決めるためにx,yのmax,minを出す
    let mut x_min=i32::MAX;
    let mut x_max=i32::MIN;
    let mut y_min=i32::MAX;
    let mut y_max=i32::MIN;

    let mut query_rows: Vec<(i32, i32)> = Vec::new();
    for row in rows_data {
        let x_value_opt: Option<i32> = row.try_get(0).ok().flatten();
        let y_value_opt: Option<i32> = row.try_get(1).ok().flatten();
        if let (Some(x_value), Some(y_value)) = (x_value_opt, y_value_opt) {
            if x_value < x_min { x_min = x_value; }
            if x_value > x_max { x_max = x_value; }
            if y_value < y_min { y_min = y_value; }
            if y_value > y_max { y_max = y_value; }
            query_rows.push((x_value, y_value));
        }
    }

    //グリッド幅を計算
    let grid_len_x=(x_max as f64-x_min as f64)/graph_condition.bins_x as f64;
    let grid_len_y=(y_max as f64-y_min as f64)/graph_condition.bins_y as f64;

    //グリッド毎の数量を初期化
    let mut arr = vec![vec![0; graph_condition.bins_y as usize]; graph_condition.bins_x as usize];

    for (x_val, y_val) in query_rows.iter() {
        let grid_num_x = (((x_val - x_min) as f64 / grid_len_x) as usize)
            .min(graph_condition.bins_x as usize - 1);
        let grid_num_y = (((y_val - y_min) as f64 / grid_len_y) as usize)
            .min(graph_condition.bins_y as usize - 1);

        arr[grid_num_x][grid_num_y] += 1;
    }

    //HashmapにVec<PlotData>でまとめる
    let rows= data_map.get_mut("data").unwrap();
    for y in 0..graph_condition.bins_y{
        for x in 0..graph_condition.bins_x{
            rows.push(PlotData::Heatmap(HeatmapData{x_data:x,y_data:y,z_data:Some(arr[x as usize][y as usize])}));
        }
    }

    Ok(GridData { grid_x: grid_len_x, grid_y: grid_len_y, x_min: x_min, y_min: y_min, histogram_bin_info: None })
}
