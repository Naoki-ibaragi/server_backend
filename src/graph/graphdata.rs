/* グラフ描画用のデータを取得するクレート */
use rusqlite::{Connection, Result};
use std::error::Error;
use std::collections::HashMap;

//独自クレートのimport
use crate::graph::variants::*;
use crate::graph::sql::{create_alarm_sql,create_sql};
use crate::graph::alarm_plotdata::*;
use crate::graph::plotdata::*;

//DBからデータを取得してHighChartで使用可能なデータに成形する
pub fn get_graphdata_from_db(db_path:&str,graph_condition:&GraphCondition)->Result<(HashMap<String,Vec<PlotData>>,GridData),Box<dyn Error>>{
    //DBに接続
    let conn=Connection::open(db_path);

    //接続に成功すればdbにConnectionを格納する
    let db=match conn{
        Ok(db)=>db,
        Err(e)=>return Err(Box::new(e)),
    };

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
        let sql = create_sql(&graph_condition, table_name);
        union_sql_parts.push(sql);
    }

    //全テーブルのデータを統合
    let sql = union_sql_parts.join(" UNION ALL ");
    println!("=== UNION ALL SQL ===");
    println!("{}", sql);
    println!("====================");

    // --- 件数を先に取得 ---
    let count_sql = format!(
        "SELECT COUNT(*) FROM ({}) AS subquery",
        sql
    );
    println!("sql:{}",count_sql);
    let total_count: i64 = db.query_row(&count_sql, [], |row| row.get(0))?;
    println!("total count:{}",total_count);

    let mut stmt=db.prepare(&sql)?;
    println!("Statement prepared successfully");

    // デバッグ: 直接クエリを実行してみる
    let mut test_rows = stmt.query([])?;
    let mut test_count = 0;
    while let Some(_row) = test_rows.next()? {
        test_count += 1;
    }
    println!("Direct query test: {} rows found", test_count);
    drop(test_rows);

    // Statementをリセット
    let mut stmt=db.prepare(&sql)?;

    //ここにHighChartsで表示用のデータを全て入れる
    let mut data_map:HashMap<String,Vec<PlotData>>=HashMap::new();
    let mut grid_data=GridData{grid_x:0.,grid_y:0.,x_min:0,y_min:0};

    //グラフ種類ごとにデータを格納
    match graph_condition.plot_unit.as_str() {
        "None" => match graph_condition.graph_type.as_str() {
            "ScatterPlot" => plot_scatterplot_without_unit( total_count, &mut data_map, &mut stmt,&graph_condition)?,
            "LinePlot" => plot_scatterplot_without_unit(total_count, &mut data_map, &mut stmt, &graph_condition)?,
            "Histogram" => plot_histogram_without_unit(total_count, &mut data_map, &mut stmt, &graph_condition)?,
            "DensityPlot" => {
                let grid_data:GridData = plot_densityplot_without_unit(total_count, &mut data_map, &mut stmt, &graph_condition)?;
            },
            _ => {},
        },
        _ => match graph_condition.graph_type.as_str() {
            "ScatterPlot" => plot_scatterplot_with_unit(total_count, &mut data_map, &mut stmt,&graph_condition)?,
            "LinePlot" => plot_scatterplot_with_unit(total_count, &mut data_map, &mut stmt,&graph_condition)?,
            "Histogram" => plot_histogram_with_unit(total_count, &mut data_map, &mut stmt, &graph_condition)?,
            _ => {},
        },
    };

    //アラームのプロットを重ねる場合の処理を入れる
    if !graph_condition.alarm.codes.is_empty(){

        //各テーブルからアラームデータを取得するためのUNION ALL SQLを生成
        let mut alarm_union_sql_parts = Vec::new();
        for table_name in &table_names {
            let sql = create_alarm_sql(&graph_condition, table_name);
            alarm_union_sql_parts.push(sql);
        }

        //全テーブルのアラームデータを統合
        let sql = alarm_union_sql_parts.join(" UNION ALL ");

        // --- 件数を先に取得 ---
        let count_sql = format!(
            "SELECT COUNT(*) FROM ({}) AS subquery",
            sql
        );
        let total_count: i64 = db.query_row(&count_sql, [], |row| row.get(0))?;

        let mut stmt=db.prepare(&sql)?;

        //アラーム分のデータをdata_mapに追加する
        match graph_condition.plot_unit.as_str() {
            "None" => match graph_condition.graph_type.as_str() { //ユニット毎にデータをまとめない
                "ScatterPlot" => plot_scatterplot_without_unit_only_alarm_data(total_count, &mut data_map, &mut stmt,&graph_condition)?,
                "LinePlot" => plot_scatterplot_without_unit_only_alarm_data(total_count, &mut data_map, &mut stmt,&graph_condition)?,
                "Histogram" => plot_histogram_without_unit_only_alarm_data(total_count, &mut data_map, &mut stmt)?,
                _ => {},
            },
            _ => match graph_condition.graph_type.as_str() { //ユニット毎にデータをまとめる
                "ScatterPlot" => plot_scatterplot_with_unit_only_alarm_data(total_count, &mut data_map, &mut stmt,&graph_condition)?,
                "LinePlot" => plot_scatterplot_with_unit_only_alarm_data(total_count, &mut data_map, &mut stmt,&graph_condition)?,
                "Histogram" => plot_histogram_with_unit_only_alarm_data(total_count, &mut data_map, &mut stmt)?,
                _ => {},
            },
        };
    }

    //SQL文を定義
    Ok((data_map,grid_data))

}
