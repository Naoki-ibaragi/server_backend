/* グラフ描画用のデータを取得するクレート */
use sqlx::{PgPool, Row,Column};
use std::error::Error;
use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, info};

//独自クレートのimport
use crate::graph::variants::*;
use crate::graph::sql::{create_alarm_sql,create_sql};
use crate::graph::alarm_plotdata::*;
use crate::graph::plotdata::*;

//DBからデータを取得してHighChartで使用可能なデータに成形する
pub async fn get_graphdata_from_db(pool:&PgPool,graph_condition:&GraphCondition)->Result<(HashMap<String,Vec<PlotData>>,GridData),Box<dyn Error>>{
    // プールから接続を使用（自動的に管理される）

    //sql文を作成（パラメータ化）
    let (mut sql, params) = create_sql(&graph_condition)
        .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, e)) as Box<dyn Error>)?;

    // LinePlotの場合はUNION ALL全体に対してORDER BYを追加
    if graph_condition.graph_type == "LinePlot" {
        sql += " ORDER BY LD_PICKUP_DATE ASC";
    }

    debug!("Generated SQL: {}", sql);

    // --- 件数を先に取得 ---
    let count_sql = format!(
        "SELECT COUNT(*) FROM ({}) AS subquery",
        sql
    );
    debug!("Count SQL: {}", count_sql);

    // パラメータをバインド
    let mut count_query = sqlx::query_scalar(&count_sql);
    for param in &params {
        count_query = count_query.bind(param);
    }

    let total_count: i64 = count_query.fetch_one(pool).await?;
    debug!("Total count: {}", total_count);

    //ここにHighChartsで表示用のデータを全て入れる
    let mut data_map:HashMap<String,Vec<PlotData>>=HashMap::new();
    let mut grid_data=GridData{grid_x:0.,grid_y:0.,x_min:0,y_min:0,histogram_bin_info:None};

    let start=Instant::now();

    //グラフ種類ごとにデータを格納
    match graph_condition.plot_unit.as_str() {
        "None" => match graph_condition.graph_type.as_str() {
            "ScatterPlot" => plot_scatterplot_without_unit(total_count, &mut data_map, pool, &sql, &params, &graph_condition).await?,
            "LinePlot" => plot_lineplot_without_unit(total_count, &mut data_map, pool, &sql, &params, &graph_condition).await?,
            "Histogram" => {
                grid_data.histogram_bin_info=Some(plot_histogram_without_unit(total_count, &mut data_map, pool, &sql, &params, &graph_condition).await?);
            }
            "DensityPlot" => {
                grid_data = plot_densityplot_without_unit(total_count, &mut data_map, pool, &sql, &params, &graph_condition).await?;
            },
            _ => {},
        },
        _ => match graph_condition.graph_type.as_str() {
            "ScatterPlot" => plot_scatterplot_with_unit(total_count, &mut data_map, pool, &sql, &params, &graph_condition).await?,
            "LinePlot" => plot_lineplot_with_unit(total_count, &mut data_map, pool, &sql, &params, &graph_condition).await?,
            "Histogram" => {
                grid_data.histogram_bin_info=Some(plot_histogram_with_unit(total_count, &mut data_map, pool, &sql, &params, &graph_condition).await?);
            }
            _ => {},
        },
    };

    let duration=start.elapsed();
    info!("Graph data processing time: {:?}", duration);

    //アラームのプロットを重ねる場合の処理を入れる
    if !graph_condition.alarm.codes.is_empty() && graph_condition.graph_type!="LinePlot" {

        //アラームデータ取得用のSQL文を生成（パラメータ化）
        let (mut alarm_sql, alarm_params) = create_alarm_sql(&graph_condition)
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, e)) as Box<dyn Error>)?;

        // LinePlotの場合はORDER BYを追加
        if graph_condition.graph_type == "LinePlot" {
            alarm_sql += " ORDER BY LD_PICKUP_DATE ASC";
        }

        // --- 件数を先に取得 ---
        let count_sql = format!(
            "SELECT COUNT(*) FROM ({}) AS subquery",
            alarm_sql
        );

        let mut alarm_count_query = sqlx::query_scalar(&count_sql);
        for param in &alarm_params {
            alarm_count_query = alarm_count_query.bind(param);
        }
        let total_count: i64 = alarm_count_query.fetch_one(pool).await?;

        //アラーム分のデータをdata_mapに追加する
        match graph_condition.plot_unit.as_str() {
            "None" => match graph_condition.graph_type.as_str() { //ユニット毎にデータをまとめない
                "Histogram" => {
                    if let Some(ref bin_info) = grid_data.histogram_bin_info {
                        plot_histogram_without_unit_only_alarm_data(total_count, &mut data_map, pool, &alarm_sql, &alarm_params, bin_info).await?;
                    }
                },
                _ => {},
            },
            _ => match graph_condition.graph_type.as_str() { //ユニット毎にデータをまとめる
                "Histogram" => {
                    if let Some(ref bin_info) = grid_data.histogram_bin_info {
                        plot_histogram_with_unit_only_alarm_data(total_count, &mut data_map, pool, &alarm_sql, &alarm_params, bin_info).await?;
                    }
                },
                _ => {},
            },
        };
    }

    Ok((data_map,grid_data))

}
