use crate::graph::variants::*;

// グラフ条件から適切なSQL文を作成
pub fn create_sql(graph_condition: &GraphCondition, table_name: &str) -> String {
    let mut sql = String::from("SELECT ");

    // X, Yデータ取得
    if graph_condition.graph_type == "LinePlot" || graph_condition.graph_type=="ScatterPlot" || graph_condition.graph_type=="DensityPlot"{
        //プロット単位をまとめるかどうかで決める
        if graph_condition.plot_unit=="None" {
            sql += &format!(
                "{}, {} FROM {} WHERE LD_PICKUP_DATE > '{}' AND ULD_PUT_DATE < '{}'",
                graph_condition.graph_x_item,
                graph_condition.graph_y_item,
                table_name,
                graph_condition.start_date,
                graph_condition.end_date
            );
        }else{
            sql += &format!(
                "{}, {}, {} FROM {} WHERE LD_PICKUP_DATE > '{}' AND ULD_PUT_DATE < '{}'",
                graph_condition.plot_unit,
                graph_condition.graph_x_item,
                graph_condition.graph_y_item,
                table_name,
                graph_condition.start_date,
                graph_condition.end_date
            );
        }
    }else if graph_condition.graph_type=="Histogram"{
        //プロット単位をまとめるかどうかで決める
        if graph_condition.plot_unit=="None" {
            sql += &format!(
                "{} FROM {} WHERE LD_PICKUP_DATE > '{}' AND ULD_PUT_DATE < '{}'",
                graph_condition.graph_x_item,
                table_name,
                graph_condition.start_date,
                graph_condition.end_date
            );
        }else{
            sql += &format!(
                "{}, {} FROM {} WHERE LD_PICKUP_DATE > '{}' AND ULD_PUT_DATE < '{}'",
                graph_condition.plot_unit,
                graph_condition.graph_x_item,
                table_name,
                graph_condition.start_date,
                graph_condition.end_date
            );
        }

    }

    // フィルター情報追加
    if !graph_condition.filters.is_empty() {
        sql += " AND ";
        for (index, filter) in graph_condition.filters.iter().enumerate() {
            let item = &filter.item;
            let value = &filter.value;
            let comparison = &filter.comparison;

            sql += &format!("{} {} '{}'", item, comparison, value);

            if index + 1 < graph_condition.filters.len() {
                sql += &format!(" {} ", graph_condition.filter_conjunction);
            }
        }
    }

    println!("{}",sql);

    sql
}

// アラーム条件にあうレコードのみ取得すようなSQL文を作成
pub fn create_alarm_sql(graph_condition: &GraphCondition, table_name: &str) -> String {
    let mut sql = String::from("SELECT ");

    // プロットデータ取得用のSQLを定義
    if graph_condition.graph_type == "LinePlot" || graph_condition.graph_type=="ScatterPlot" || graph_condition.graph_type=="DensityPlot" {
        //プロット単位をまとめるかどうかで決める
        if graph_condition.plot_unit=="None" {
            sql += &format!(
                "{}, {} FROM {} WHERE LD_PICKUP_DATE > '{}' AND ULD_PUT_DATE < '{}'",
                graph_condition.graph_x_item,
                graph_condition.graph_y_item,
                table_name,
                graph_condition.start_date,
                graph_condition.end_date
            );
        }else{
            sql += &format!(
                "{}, {}, {} FROM {} WHERE LD_PICKUP_DATE > '{}' AND ULD_PUT_DATE < '{}'",
                graph_condition.plot_unit,
                graph_condition.graph_x_item,
                graph_condition.graph_y_item,
                table_name,
                graph_condition.start_date,
                graph_condition.end_date
            );
        }
    }else if graph_condition.graph_type=="Histogram"{
        //プロット単位をまとめるかどうかで決める
        if graph_condition.plot_unit=="None" {
            sql += &format!(
                "{} FROM {} WHERE LD_PICKUP_DATE > '{}' AND ULD_PUT_DATE < '{}'",
                graph_condition.graph_x_item,
                table_name,
                graph_condition.start_date,
                graph_condition.end_date
            );
        }else{
            sql += &format!(
                "{}, {} FROM {} WHERE LD_PICKUP_DATE > '{}' AND ULD_PUT_DATE < '{}'",
                graph_condition.plot_unit,
                graph_condition.graph_x_item,
                table_name,
                graph_condition.start_date,
                graph_condition.end_date
            );
        }

    }

    //アラームフィルター追加
    sql+=" AND ";
    let alarm_column=format!("{}_ALARM",graph_condition.alarm.unit);
    for alarm_code in graph_condition.alarm.codes.iter(){
        sql += &format!("{} {} '{}'",alarm_column,"=",alarm_code);

    }

    // フィルター情報追加
    if !graph_condition.filters.is_empty() {
        sql += " AND ";
        for (index, filter) in graph_condition.filters.iter().enumerate() {
            let item = &filter.item;
            let value = &filter.value;
            let comparison = &filter.comparison;

            sql += &format!("{} {} '{}'", item, comparison, value);

            if index + 1 < graph_condition.filters.len() {
                sql += &format!(" {} ", graph_condition.filter_conjunction);
            }
        }
    }

    println!("{}",sql);

    sql
}
