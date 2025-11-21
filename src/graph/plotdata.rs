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

/* Heatmap(Histogram) */
//プロット分割しないヒストグラムータを取得
pub fn plot_histogram_without_unit(total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,stmt:&mut Statement,_graph_condition:&GraphCondition)->Result<(),Box<dyn Error>>{
    //dataキーに値を入れる
    data_map.entry("data".to_string()).or_insert(vec![]);

    let query_rows: Vec<i32> = stmt.query_map([], |row| {
        let x_value: i32 = row.get(0)?;
        Ok(x_value)
    })?
    .filter_map(|r| r.ok())
    .collect();

    // 最初に全ての行をカウント（オプション：パフォーマンスが心配な場合は別途COUNT(*)で取得）
    // 以下のコードでは処理しながら報告していく方式を使用
    let rows= data_map.get_mut("data").unwrap();
    for (index,data) in query_rows.into_iter().enumerate(){
        rows.push(PlotData::Histogram(HistogramData{x_data:data}));

        // 1000行ごとに進捗を報告
    }

    Ok(())
}

//プロット分割するヒストグラムのデータを取得
pub fn plot_histogram_with_unit(total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,stmt:&mut Statement,_graph_condition:&GraphCondition)->Result<(),Box<dyn Error>>{
    let query_rows: Vec<(String,i32)> = stmt.query_map([], |row| {
        let unit_name: String=row.get(0)?;
        let x_value: i32 = row.get(1)?;
        Ok((unit_name,x_value))
    })?
    .filter_map(|r| r.ok())
    .collect();

    for (index, record) in query_rows.into_iter().enumerate(){
        data_map.entry(record.0).or_insert(vec![]).push(
            PlotData::Histogram(HistogramData{x_data:record.1})
        );
    }

    Ok(())
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

    Ok(GridData { grid_x: grid_len_x, grid_y: grid_len_y, x_min: x_min, y_min: y_min })
}

