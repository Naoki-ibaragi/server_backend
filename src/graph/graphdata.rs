/* グラフ描画用のデータを取得するクレート */
use sqlx::{PgPool, Row,Column};
use std::error::Error;
use std::collections::HashMap;
use std::time::Instant;

//独自クレートのimport
use crate::graph::variants::*;
use crate::graph::sql::{create_alarm_sql,create_sql};
use crate::graph::alarm_plotdata::*;
use crate::graph::plotdata::*;

//DBからデータを取得してHighChartで使用可能なデータに成形する
pub async fn get_graphdata_from_db(database_url:&str,graph_condition:&GraphCondition)->Result<(HashMap<String,Vec<PlotData>>,GridData),Box<dyn Error>>{
    let pool = PgPool::connect(database_url).await?;

    //sql文を作成
    let mut sql = create_sql(&graph_condition);

    // LinePlotの場合はUNION ALL全体に対してORDER BYを追加
    if graph_condition.graph_type == "LinePlot" {
        sql += " ORDER BY LD_PICKUP_DATE ASC";
    }

    println!("sql:{}", sql);

    // --- 件数を先に取得 ---
    let count_sql = format!(
        "SELECT COUNT(*) FROM ({}) AS subquery",
        sql
    );
    println!("count_sql:{}",count_sql);

    let total_count: i64 = sqlx::query_scalar(&count_sql)
        .fetch_one(&pool)
        .await?;
    println!("total count:{}",total_count);

    //ここにHighChartsで表示用のデータを全て入れる
    let mut data_map:HashMap<String,Vec<PlotData>>=HashMap::new();
    let mut grid_data=GridData{grid_x:0.,grid_y:0.,x_min:0,y_min:0,histogram_bin_info:None};

    let start=Instant::now();

    //グラフ種類ごとにデータを格納
    match graph_condition.plot_unit.as_str() {
        "None" => match graph_condition.graph_type.as_str() {
            "ScatterPlot" => plot_scatterplot_without_unit(total_count, &mut data_map, &pool, &sql, &graph_condition).await?,
            "LinePlot" => plot_lineplot_without_unit(total_count, &mut data_map, &pool, &sql, &graph_condition).await?,
            "Histogram" => {
                grid_data.histogram_bin_info=Some(plot_histogram_without_unit(total_count, &mut data_map, &pool, &sql, &graph_condition).await?);
            }
            "DensityPlot" => {
                grid_data = plot_densityplot_without_unit(total_count, &mut data_map, &pool, &sql, &graph_condition).await?;
            },
            _ => {},
        },
        _ => match graph_condition.graph_type.as_str() {
            "ScatterPlot" => plot_scatterplot_with_unit(total_count, &mut data_map, &pool, &sql, &graph_condition).await?,
            "LinePlot" => plot_lineplot_with_unit(total_count, &mut data_map, &pool, &sql, &graph_condition).await?,
            "Histogram" => {
                grid_data.histogram_bin_info=Some(plot_histogram_with_unit(total_count, &mut data_map, &pool, &sql, &graph_condition).await?);
            }
            _ => {},
        },
    };

    let duration=start.elapsed();
    println!("処理時間:{:?}",duration);

    //アラームのプロットを重ねる場合の処理を入れる
    if !graph_condition.alarm.codes.is_empty() && graph_condition.graph_type!="LinePlot" {

        //アラームデータ取得用のSQL文を生成
        let mut alarm_sql = create_alarm_sql(&graph_condition);

        // LinePlotの場合はORDER BYを追加
        if graph_condition.graph_type == "LinePlot" {
            alarm_sql += " ORDER BY LD_PICKUP_DATE ASC";
        }

        // --- 件数を先に取得 ---
        let count_sql = format!(
            "SELECT COUNT(*) FROM ({}) AS subquery",
            alarm_sql
        );
        let total_count: i64 = sqlx::query_scalar(&count_sql)
            .fetch_one(&pool)
            .await?;

        //アラーム分のデータをdata_mapに追加する
        match graph_condition.plot_unit.as_str() {
            "None" => match graph_condition.graph_type.as_str() { //ユニット毎にデータをまとめない
                "Histogram" => {
                    if let Some(ref bin_info) = grid_data.histogram_bin_info {
                        plot_histogram_without_unit_only_alarm_data(total_count, &mut data_map, &pool, &alarm_sql, bin_info).await?;
                    }
                },
                _ => {},
            },
            _ => match graph_condition.graph_type.as_str() { //ユニット毎にデータをまとめる
                "Histogram" => {
                    if let Some(ref bin_info) = grid_data.histogram_bin_info {
                        plot_histogram_with_unit_only_alarm_data(total_count, &mut data_map, &pool, &alarm_sql, bin_info).await?;
                    }
                },
                _ => {},
            },
        };
    }

    Ok((data_map,grid_data))

}
