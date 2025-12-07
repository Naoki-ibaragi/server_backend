use crate::graph::{self, variants::*};

// グラフ条件から適切なSQL文を作成
pub fn create_sql(graph_condition: &GraphCondition) -> String {
    let mut sql = String::from("SELECT ");

    // X, Yデータ取得
    if graph_condition.graph_type=="ScatterPlot" || graph_condition.graph_type=="DensityPlot"{
        //プロット単位をまとめるかどうかで決める
        if graph_condition.plot_unit=="None" {
            sql += &format!(
                "{}, {} FROM chipdata",
                graph_condition.graph_x_item,
                graph_condition.graph_y_item,
            );
        }else{
            sql += &format!(
                "{}, {}, {} FROM chipdata",
                graph_condition.plot_unit,
                graph_condition.graph_x_item,
                graph_condition.graph_y_item,
            );
        }
    }else if graph_condition.graph_type=="Histogram"{
        //プロット単位をまとめるかどうかで決める
        if graph_condition.plot_unit=="None" {
            sql += &format!(
                "{} FROM chipdata",
                graph_condition.graph_x_item,
            );
        }else{
            sql += &format!(
                "{}, {} FROM chipdata",
                graph_condition.plot_unit,
                graph_condition.graph_x_item,
            );
        }
    }else if graph_condition.graph_type=="LinePlot"{ //時系列プロットの場合はx軸は必ずLD_PICKUP_DATEをとり、アラームが設定されていればそれも取る
        if graph_condition.alarm.codes.is_empty(){ //アラームプロットを重ねない場合
            if graph_condition.plot_unit=="None" {
                sql += &format!(
                    "{}, {} FROM chipdata",
                    "LD_PICKUP_DATE",
                    graph_condition.graph_y_item,
                );
            }else{
                sql += &format!(
                    "{}, {}, {} FROM chipdata",
                    graph_condition.plot_unit,
                    "LD_PICKUP_DATE",
                    graph_condition.graph_y_item,
                );
            }
        }else{//アラームプロットを重ねる場合
            if graph_condition.plot_unit=="None" {
                sql += &format!(
                    "{}, {}, {} FROM chipdata",
                    "LD_PICKUP_DATE",
                    graph_condition.graph_y_item,
                    graph_condition.alarm.unit.clone()+"_ALARM",
                );
            }else{
                sql += &format!(
                    "{}, {}, {}, {} FROM chipdata",
                    graph_condition.plot_unit,
                    "LD_PICKUP_DATE",
                    graph_condition.graph_y_item,
                    graph_condition.alarm.unit.clone()+"_ALARM",
                );
            }
        }
    }

    // フィルター情報追加
    if !graph_condition.filters.is_empty() {
        sql += " WHERE ";
        for (index, filter) in graph_condition.filters.iter().enumerate() {
            let item = &filter.item;
            let value = &filter.value;
            let comparison = &filter.comparison;

            if comparison=="LIKE"{
                println!("1.{}",comparison);
                sql += &format!("{} {} '%{}%'", item, comparison, value);
            }else{
                println!("2.{}",comparison);
                sql += &format!("{} {} '{}'", item, comparison, value);
            }

            if index + 1 < graph_condition.filters.len() {
                sql += &format!(" {} ", graph_condition.filter_conjunction);
            }
        }
        //パーティション情報追加
        sql+=&format!(" AND ld_pickup_date BETWEEN '{}' AND '{}'",graph_condition.start_date,graph_condition.end_date);
    } else {
        //パーティション情報追加
        sql+=&format!(" WHERE ld_pickup_date BETWEEN '{}' AND '{}'",graph_condition.start_date,graph_condition.end_date);
    }


    println!("{}",sql);

    sql
}

// アラーム条件にあうレコードのみ取得すようなSQL文を作成
pub fn create_alarm_sql(graph_condition: &GraphCondition) -> String {
    let mut sql = String::from("SELECT ");

    // プロットデータ取得用のSQLを定義
    if graph_condition.graph_type == "LinePlot" {
        //LinePlotの場合はx軸は必ずLD_PICKUP_DATE
        if graph_condition.plot_unit=="None" {
            sql += &format!(
                "{}, {} FROM chipdata",
                "LD_PICKUP_DATE",
                graph_condition.graph_y_item,
            );
        }else{
            sql += &format!(
                "{}, {}, {} FROM chipdata",
                graph_condition.plot_unit,
                "LD_PICKUP_DATE",
                graph_condition.graph_y_item,
            );
        }
    }else if graph_condition.graph_type=="ScatterPlot" || graph_condition.graph_type=="DensityPlot" {
        //プロット単位をまとめるかどうかで決める
        if graph_condition.plot_unit=="None" {
            sql += &format!(
                "{}, {} FROM chipdata",
                graph_condition.graph_x_item,
                graph_condition.graph_y_item,
            );
        }else{
            sql += &format!(
                "{}, {}, {} FROM chipdata",
                graph_condition.plot_unit,
                graph_condition.graph_x_item,
                graph_condition.graph_y_item,
            );
        }
    }else if graph_condition.graph_type=="Histogram"{
        //プロット単位をまとめるかどうかで決める
        if graph_condition.plot_unit=="None" {
            sql += &format!(
                "{} FROM chipdata",
                graph_condition.graph_x_item,
            );
        }else{
            sql += &format!(
                "{}, {} FROM chipdata",
                graph_condition.plot_unit,
                graph_condition.graph_x_item,
            );
        }

    }

    //アラームフィルター追加
    sql+=" WHERE ";
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

    //パーティション情報追加
    sql+=&format!(" AND ld_pickup_date BETWEEN '{}' AND '{}'",graph_condition.start_date,graph_condition.end_date);

    println!("{}",sql);

    sql
}
