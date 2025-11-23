use rusqlite::{Statement};
use std::error::Error;
use std::collections::HashMap;

use crate::graph::variants::*;

//プロット分割しない散布図のデータを取得
pub fn plot_scatterplot_without_unit(total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,stmt:&mut Statement,graph_condition:&GraphCondition)->Result<(),Box<dyn Error>>{
    data_map.entry("data".to_string()).or_insert(vec![]);

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
    let rows= data_map.get_mut("data").unwrap();
    for (_index,record) in query_rows.into_iter().enumerate(){
        rows.push(PlotData::Scatter(ScatterPlotData{x_data:record.0,y_data:record.1}));
    }

    println!("Final data_map size: {}", rows.len());

    Ok(())
}

//プロット分割する散布図のデータを取得
pub fn plot_scatterplot_with_unit(total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,stmt:&mut Statement,graph_condition:&GraphCondition)->Result<(),Box<dyn Error>>{
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
        data_map.entry(record.0).or_insert(vec![]).push(
            PlotData::Scatter(ScatterPlotData{x_data:record.1, y_data:record.2})
        );
    }
    Ok(())
}

//プロット分割しない折れ線グラフ(時系列プロット)のデータを取得
//LD_PICKUP_DATEでORDERされた状態でデータ取得済
pub fn plot_lineplot_without_unit(total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,stmt:&mut Statement,graph_condition:&GraphCondition)->Result<(),Box<dyn Error>>{
    data_map.entry("data".to_string()).or_insert(vec![]);

    if graph_condition.alarm.codes.is_empty(){ //アラーム情報を取得しない場合
        let query_rows: Vec<i32> = stmt.query_map([], |row| {
            let y_value: i32 = row.get(1)?;
            Ok(y_value)
        })?
        .filter_map(|r| r.ok())
        .collect();

        println!("query_rows collected: {} rows", query_rows.len());

        // 最初に全ての行をカウント（オプション：パフォーマンスが心配な場合は別途COUNT(*)で取得）
        // 以下のコードでは処理しながら報告していく方式を使用
        let rows= data_map.get_mut("data").unwrap();
        for (_index,record) in query_rows.into_iter().enumerate(){
            rows.push(PlotData::Line(LinePlotData{y_data:record,is_alarm:false}));
        }

        println!("Final data_map size: {}", rows.len());

        Ok(())
    }else{
        let target_alarm_code:Vec<i32>=graph_condition.alarm.codes.clone(); //集計対象のアラームコードリスト
        let query_rows: Vec<(i32,bool)> = stmt.query_map([], |row| {
            let y_value: i32 = row.get(1)?;
            let alarm_value:i32=row.get(2)?;
            if target_alarm_code.contains(&alarm_value){
                Ok((y_value,true))
            }else{
                Ok((y_value,false))
            }
        })?
        .filter_map(|r| r.ok())
        .collect();

        println!("query_rows collected: {} rows", query_rows.len());

        // 最初に全ての行をカウント（オプション：パフォーマンスが心配な場合は別途COUNT(*)で取得）
        // 以下のコードでは処理しながら報告していく方式を使用
        let rows= data_map.get_mut("data").unwrap();
        for (_index,record) in query_rows.into_iter().enumerate(){
            rows.push(PlotData::Line(LinePlotData{y_data:record.0,is_alarm:record.1}));
        }

        println!("Final data_map size: {}", rows.len());

        Ok(())
    }
}

//プロット分割する散布図のデータを取得
pub fn plot_lineplot_with_unit(total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,stmt:&mut Statement,graph_condition:&GraphCondition)->Result<(),Box<dyn Error>>{

    if graph_condition.alarm.codes.is_empty(){ //アラーム情報を取得しない場合
        let query_rows: Vec<(String,i32)> = stmt.query_map([], |row| {
            let unit: String = row.get(0)?;
            let y_value: i32 = row.get(2)?;
            Ok((unit,y_value))
        })?
        .filter_map(|r| r.ok())
        .collect();

        for (index, record) in query_rows.into_iter().enumerate(){
            data_map.entry(record.0).or_insert(vec![]).push(
                PlotData::Line(LinePlotData{y_data:record.1,is_alarm:false})
            );
        }
        Ok(())
    }else{
        let target_alarm_code:Vec<i32>=graph_condition.alarm.codes.clone(); //集計対象のアラームコードリスト
        let query_rows: Vec<(String,i32,bool)> = stmt.query_map([], |row| {
            let unit: String = row.get(0)?;
            let y_value: i32 = row.get(2)?;
            let alarm_value:i32=row.get(3)?;
            if target_alarm_code.contains(&alarm_value){
                Ok((unit,y_value,true))
            }else{
                Ok((unit,y_value,false))
            }
        })?
        .filter_map(|r| r.ok())
        .collect();

        for (index, record) in query_rows.into_iter().enumerate(){
            data_map.entry(record.0).or_insert(vec![]).push(
                PlotData::Line(LinePlotData{y_data:record.1,is_alarm:record.2})
            );
        }
        Ok(())
    }
}

/* Heatmap(Histogram) */
//プロット分割しないヒストグラムータを取得
pub fn plot_histogram_without_unit(_total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,stmt:&mut Statement,graph_condition:&GraphCondition)->Result<HistogramBinInfo,Box<dyn Error>>{
    //dataキーに値を入れる
    data_map.entry("data".to_string()).or_insert(vec![]);

    let query_rows: Vec<i32> = stmt.query_map([], |row| {
        let x_value: i32 = row.get(0)?;
        Ok(x_value)
    })?
    .filter_map(|r| r.ok())
    .collect();

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
pub fn plot_histogram_with_unit(_total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,stmt:&mut Statement,graph_condition:&GraphCondition)->Result<HistogramBinInfo,Box<dyn Error>>{
    let query_rows: Vec<(String,i32)> = stmt.query_map([], |row| {
        let unit_name: String=row.get(0)?;
        let x_value: i32 = row.get(1)?;
        Ok((unit_name,x_value))
    })?
    .filter_map(|r| r.ok())
    .collect();

    if query_rows.is_empty(){
        return Ok(HistogramBinInfo{
            bin_edges: vec![],
            bin_width: 0.0,
        });
    }

    // 全データの最小値と最大値を取得（全ユニットで同じビンを使用）
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
pub fn plot_densityplot_without_unit(total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,stmt:&mut Statement,graph_condition:&GraphCondition)->Result<GridData,Box<dyn Error>>{
    data_map.entry("data".to_string()).or_insert(vec![]);

    //格格幅を決めるためにx,yのmax,minを出す
    let mut x_min=i32::MAX;
    let mut x_max=i32::MIN;
    let mut y_min=i32::MAX;
    let mut y_max=i32::MIN;

    let query_rows: Vec<(i32, i32)> = stmt.query_map([], |row| {
    let x_value: i32 = row.get(0)?;
    let y_value: i32 = row.get(1)?;
    Ok((x_value, y_value))
    })?
    .filter_map(|r| {
        let (x_val, y_val) = r.ok()?;
        if x_val < x_min { x_min = x_val; }
        if x_val > x_max { x_max = x_val; }
        if y_val < y_min { y_min = y_val; }
        if y_val > y_max { y_max = y_val; }
        Some((x_val, y_val))
    })
    .collect();

    //グリッド幅を計算
    let grid_len_x=(x_max as f64-x_min as f64)/graph_condition.bins_x as f64;
    let grid_len_y=(y_max as f64-y_min as f64)/graph_condition.bins_y as f64;

    //グリッド毎の数量を初期化
    let mut arr = vec![vec![0; graph_condition.bins_y as usize]; graph_condition.bins_x as usize];

    for (index, (x_val, y_val)) in query_rows.iter().enumerate() {
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
            rows.push(PlotData::Heatmap(HeatmapData{x_data:x,y_data:y,z_data:arr[x as usize][y as usize]}));
        }
    }

    Ok(GridData { grid_x: grid_len_x, grid_y: grid_len_y, x_min: x_min, y_min: y_min, histogram_bin_info: None })
}

