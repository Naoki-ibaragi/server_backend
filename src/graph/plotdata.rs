use rusqlite::{Statement};
use std::error::Error;
use std::collections::HashMap;

use crate::graph::variants::*;

//プロット分割しない散布図のデータを取得
pub fn plot_scatterplot_without_unit(total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,stmt:&mut Statement)->Result<(),Box<dyn Error>>{
    println!("plot_scatterplot_without_unit - total_count: {}", total_count);
    data_map.entry("data".to_string()).or_insert(vec![]);

    let query_rows: Vec<Vec<i32>> = stmt.query_map([], |row| {
        let x_value: i32 = row.get(0)?;
        let y_value: i32 = row.get(1)?;
        Ok(vec![x_value, y_value])
    })?
    .collect::<Result<Vec<Vec<i32>>, _>>()?;

    println!("query_rows collected: {} rows", query_rows.len());

    // 最初に全ての行をカウント（オプション：パフォーマンスが心配な場合は別途COUNT(*)で取得）
    // 以下のコードでは処理しながら報告していく方式を使用
    let rows= data_map.get_mut("data").unwrap();
    for (_index,record) in query_rows.into_iter().enumerate(){
        rows.push(PlotData::Number(NumberData::new(record[0],record[1])));

    }

    println!("Final data_map size: {}", rows.len());

    Ok(())
}

//プロット分割する散布図のデータを取得
pub fn plot_scatterplot_with_unit(total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,stmt:&mut Statement)->Result<(),Box<dyn Error>>{
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
        if &record.unit!="" && data_map.contains_key(&record.unit){
            let rows=data_map.get_mut(&record.unit).unwrap();
            rows.push(match record.data{
                PlotData::Number(num_data)=>PlotData::Number(NumberData::new(num_data.x,num_data.y)),
                PlotData::Calendar(calender_data)=>PlotData::Calendar(CalenderData::new(calender_data.x,calender_data.y)),
                _=>return Err(Box::from(format!("不明なPlotData型が検出されました")))
            });
        }else{
            data_map.entry(record.unit).or_insert(
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

//プロット分割しない折れ線グラフのデータを取得
pub fn plot_lineplot_without_unit(total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,stmt:&mut Statement,graph_condition:&GraphCondition)->Result<(),Box<dyn Error>>{
    data_map.entry("data".to_string()).or_insert(vec![]);

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
    let rows= data_map.get_mut("data").unwrap();
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

//プロット分割する折れ線グラフのデータを取得
pub fn plot_lineplot_with_unit(total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,stmt:&mut Statement)->Result<(),Box<dyn Error>>{
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
        if data_map.contains_key(&record.unit){
            let rows=data_map.get_mut(&record.unit).unwrap();
            rows.push(match record.data{
                PlotData::Number(num_data)=>PlotData::Number(NumberData::new(num_data.x,num_data.y)),
                PlotData::Calendar(calender_data)=>PlotData::Calendar(CalenderData::new(calender_data.x,calender_data.y)),
                _=>return Err(Box::from(format!("不明なPlotData型が検出されました {:?}",record.data)))
            });
        }else{
            data_map.entry(record.unit).or_insert(
        match record.data{
                    PlotData::Number(num_data)=>vec![PlotData::Number(NumberData::new(num_data.x,num_data.y))],
                    PlotData::Calendar(calender_data)=>vec![PlotData::Calendar(CalenderData::new(calender_data.x,calender_data.y))],
                    _=>return Err(Box::from(format!("不明なPlotData型が検出されました {:?}",record.data)))
                }
            );
        }

        // 1000行ごとに進捗を報告
    }

    Ok(())

}

/* Heatmap(Histogram) */
//プロット分割しないヒストグラムータを取得
pub fn plot_histogram_without_unit(total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,stmt:&mut Statement,_graph_condition:&GraphCondition)->Result<(),Box<dyn Error>>{
    //dataキーに値を入れる
    data_map.entry("data".to_string()).or_insert(vec![]);

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
    let rows= data_map.get_mut("data").unwrap();
    for (index,record) in query_rows.into_iter().enumerate(){
        rows.push(PlotData::Number1D(NumberData_1D::new(record[0])));

        // 1000行ごとに進捗を報告
    }

    Ok(())
}

//プロット分割するヒストグラムのデータを取得
pub fn plot_histogram_with_unit(total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,stmt:&mut Statement,_graph_condition:&GraphCondition)->Result<(),Box<dyn Error>>{
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
        if data_map.contains_key(&record.unit){
            let rows=data_map.get_mut(&record.unit).unwrap();
            rows.push(match record.data{
                PlotData::Number1D(num_data)=>PlotData::Number1D(NumberData_1D::new(num_data.x)),
                _=>return Err(Box::from(format!("不明なPlotData型が検出されました {:?}",record.data)))
            });
        }else{
            data_map.entry(record.unit).or_insert(
        match record.data{
                    PlotData::Number1D(num_data)=>vec![PlotData::Number1D(NumberData_1D::new(num_data.x))],
                    _=>return Err(Box::from(format!("不明なPlotData型が検出されました {:?}",record.data)))
                }
            );
        }

        // 1000行ごとに進捗を報告
    }

    Ok(())
}

/* Heatmap(DensityPlot) */
//プロット分割しない密密プロットのデータを取得
pub fn plot_densityplot_without_unit(total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,stmt:&mut Statement,graph_condition:&GraphCondition)->Result<(f64,f64),Box<dyn Error>>{
    data_map.entry("data".to_string()).or_insert(vec![]);

    //格格幅を決めるためにx,yのmax,minを出す
    let mut x_min=i32::MAX;
    let mut x_max=i32::MIN;
    let mut y_min=i32::MAX;
    let mut y_max=i32::MIN;

    let query_rows: Vec<(i32, i32)> = stmt.query_map([], |row| {
    let x_value: String = row.get(0)?;
    let y_value: String = row.get(1)?;
    Ok((x_value, y_value))
    })?
    .filter_map(|r| {
        let (x_val, y_val) = r.ok()?;
        let x = x_val.parse::<i32>().ok()?;
        let y = y_val.parse::<i32>().ok()?;
        if x < x_min { x_min = x; }
        if x > x_max { x_max = x; }
        if y < y_min { y_min = y; }
        if y > y_max { y_max = y; }
        Some((x, y))
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
            rows.push(PlotData::Heatmap(HeatmapData::new(x,y,arr[x as usize][y as usize])));
        }
    }

    Ok((grid_len_x,grid_len_y))
}

//プロット分割する密度プロットのデータを取得
pub fn plot_densityplot_with_unit(_total_count:i64,data_map:&mut HashMap<String,Vec<PlotData>>,stmt:&mut Statement,graph_condition:&GraphCondition)->Result<HashMap<String,(f64,f64)>,Box<dyn Error>>{
    // ユニットごとのグリッド情報を格納
    let mut grid_info:HashMap<String,(f64,f64)>=HashMap::new();

    // まず全データを取得してユニット別に分類
    let query_rows: Vec<(String,i32,i32)> = stmt.query_map([], |row| {
        let unit_name: String=row.get(0)?;
        let x_value: String = row.get(1)?;
        let y_value: String = row.get(2)?;
        Ok((unit_name,x_value, y_value))
    })?
    .filter_map(|r| {
        let (unit_name,x_val, y_val) = r.ok()?;
        let x = x_val.parse::<i32>().ok()?;
        let y = y_val.parse::<i32>().ok()?;
        Some((unit_name,x,y))
    })
    .collect();

    // ユニット別にデータを集約
    let mut unit_data_map:HashMap<String,Vec<(i32,i32)>>=HashMap::new();
    for (unit_name,x,y) in query_rows{
        unit_data_map.entry(unit_name).or_insert(vec![]).push((x,y));
    }

    // 各ユニットごとにDensityPlot処理を実施
    for (index,(unit_name,data_points)) in unit_data_map.iter().enumerate(){
        //格子幅を決めるためにx,yのmax,minを出す
        let mut x_min=i32::MAX;
        let mut x_max=i32::MIN;
        let mut y_min=i32::MAX;
        let mut y_max=i32::MIN;

        for (x,y) in data_points{
            if *x < x_min { x_min = *x; }
            if *x > x_max { x_max = *x; }
            if *y < y_min { y_min = *y; }
            if *y > y_max { y_max = *y; }
        }

        //グリッド幅を計算
        let grid_len_x=(x_max as f64-x_min as f64)/graph_condition.bins_x as f64;
        let grid_len_y=(y_max as f64-y_min as f64)/graph_condition.bins_y as f64;

        //グリッド情報を保存
        grid_info.insert(unit_name.clone(),(grid_len_x,grid_len_y));

        //グリッド毎の数量を初期化
        let mut arr = vec![vec![0; graph_condition.bins_y as usize]; graph_condition.bins_x as usize];

        for (x_val, y_val) in data_points {
            let grid_num_x = (((*x_val - x_min) as f64 / grid_len_x) as usize)
                .min(graph_condition.bins_x as usize - 1);
            let grid_num_y = (((*y_val - y_min) as f64 / grid_len_y) as usize)
                .min(graph_condition.bins_y as usize - 1);

            arr[grid_num_x][grid_num_y] += 1;
        }

        //HashmapにVec<PlotData>でまとめる
        data_map.entry(unit_name.clone()).or_insert(vec![]);
        let rows= data_map.get_mut(unit_name).unwrap();
        for y in 0..graph_condition.bins_y{
            for x in 0..graph_condition.bins_x{
                rows.push(PlotData::Heatmap(HeatmapData::new(x,y,arr[x as usize][y as usize])));
            }
        }

        // ユニットごとに進捗を報告
    }

    Ok(grid_info)
}

