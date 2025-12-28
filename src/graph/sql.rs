use crate::graph::{self, variants::*};
use tracing::debug;

// 許可されたカラム名のリスト（ホワイトリスト）
// セキュリティのため、このリストにあるカラム名のみがSQL文で使用可能です
const ALLOWED_COLUMNS: &[&str] = &[
    // 基本カラム
    "ID", "MACHINE_ID", "TYPE_NAME", "LOT_NAME", "SERIAL", "WANO", "WAX", "WAY",

    // LD (Loader) 関連
    "LD_PICKUP_DATE", "LD_TRAYID", "LD_TRAY_ARM", "LD_TRAY_POCKET_X", "LD_TRAY_POCKET_Y",
    "LD_TRAY_ALIGN_X", "LD_TRAY_ALIGN_Y", "LD_ARM1_COLLET", "LD_ALARM",

    // DC1 (Die Checker 1) 関連
    "DC1_PRE_ALIGN_X", "DC1_PRE_ALIGN_Y", "DC1_PRE_ALIGN_T", "DC1_ARM1_COLLET",
    "DC1_STAGE_SERIAL", "DC1_STAGE_COUNT", "DC1_PROBE_SERIAL", "DC1_PROBE_COUNT",
    "DC1_PROBE_X1", "DC1_PROBE_Y1", "DC1_PROBE_X2", "DC1_PROBE_Y2",
    "DC1_STAGE_Z", "DC1_PIN_Z", "DC1_CHIP_ALIGN_X", "DC1_CHIP_ALIGN_Y", "DC1_CHIP_ALIGN_T",
    "DC1_TEST_BIN", "DC1_ARM2_COLLET", "DC1_ALARM",

    // AC1 (AC Test 1) 関連
    "AC1_ARM1_COLLET", "AC1_STAGE_SERIAL", "AC1_STAGE_COUNT", "AC1_PROBE_SERIAL", "AC1_PROBE_COUNT",
    "AC1_PROBE_X1", "AC1_PROBE_Y1", "AC1_PROBE_X2", "AC1_PROBE_Y2",
    "AC1_STAGE_Z", "AC1_PIN_Z", "AC1_CHIP_ALIGN_X", "AC1_CHIP_ALIGN_Y", "AC1_CHIP_ALIGN_T",
    "AC1_TEST_BIN", "AC1_ARM2_COLLET", "AC1_ALARM",

    // AC2 (AC Test 2) 関連
    "AC2_ARM1_COLLET", "AC2_STAGE_SERIAL", "AC2_STAGE_COUNT", "AC2_PROBE_SERIAL", "AC2_PROBE_COUNT",
    "AC2_PROBE_X1", "AC2_PROBE_Y1", "AC2_PROBE_X2", "AC2_PROBE_Y2",
    "AC2_STAGE_Z", "AC2_PIN_Z", "AC2_CHIP_ALIGN_X", "AC2_CHIP_ALIGN_Y", "AC2_CHIP_ALIGN_T",
    "AC2_TEST_BIN", "AC2_ARM2_COLLET", "AC2_ALARM",

    // DC2 (Die Checker 2) 関連
    "DC2_ARM1_COLLET", "DC2_STAGE_SERIAL", "DC2_STAGE_COUNT", "DC2_PROBE_SERIAL", "DC2_PROBE_COUNT",
    "DC2_PROBE_X1", "DC2_PROBE_Y1", "DC2_PROBE_X2", "DC2_PROBE_Y2",
    "DC2_STAGE_Z", "DC2_PIN_Z", "DC2_CHIP_ALIGN_X", "DC2_CHIP_ALIGN_Y", "DC2_CHIP_ALIGN_T",
    "DC2_TEST_BIN", "DC2_ARM2_COLLET", "DC2_ALARM",

    // IP (Inspection) 関連
    "IP_ARM1_COLLET", "IP_STAGE_COUNT", "IP_SURF_BIN", "IP_ARM2_COLLET", "IP_BACK_BIN", "IP_ALARM",

    // ULD (Unloader) 関連
    "ULD_PRE_ALIGN_X", "ULD_PRE_ALIGN_Y", "ULD_PRE_ALIGN_T", "ULD_TRAYID",
    "ULD_POCKET_X", "ULD_POCKET_Y", "ULD_POCKET_ALIGN_X", "ULD_POCKET_ALIGN_Y",
    "ULD_ARM1_COLLET", "ULD_PUT_DATE", "ULD_CHIP_ALIGN_X", "ULD_CHIP_ALIGN_Y",
    "ULD_CHIP_ALIGN_NUM", "ULD_ALARM",
];

// 許可された比較演算子のリスト
const ALLOWED_COMPARISONS: &[&str] = &["=", ">", "<", ">=", "<=", "!=", "<>", "LIKE","NOT LIKE"];

// INTEGER型のカラムのリスト
const INTEGER_COLUMNS: &[&str] = &[
    "MACHINE_ID", "SERIAL", "WANO", "WAX", "WAY",
    "LD_TRAY_POCKET_X", "LD_TRAY_POCKET_Y", "LD_TRAY_ALIGN_X", "LD_TRAY_ALIGN_Y",
    "LD_ARM1_COLLET", "LD_ALARM",
    "DC1_PRE_ALIGN_X", "DC1_PRE_ALIGN_Y", "DC1_PRE_ALIGN_T", "DC1_ARM1_COLLET",
    "DC1_STAGE_COUNT", "DC1_PROBE_COUNT", "DC1_PROBE_X1", "DC1_PROBE_Y1",
    "DC1_PROBE_X2", "DC1_PROBE_Y2", "DC1_STAGE_Z", "DC1_PIN_Z",
    "DC1_CHIP_ALIGN_X", "DC1_CHIP_ALIGN_Y", "DC1_CHIP_ALIGN_T",
    "DC1_TEST_BIN", "DC1_ARM2_COLLET", "DC1_ALARM",
    "AC1_ARM1_COLLET", "AC1_STAGE_COUNT", "AC1_PROBE_COUNT",
    "AC1_PROBE_X1", "AC1_PROBE_Y1", "AC1_PROBE_X2", "AC1_PROBE_Y2",
    "AC1_STAGE_Z", "AC1_PIN_Z", "AC1_CHIP_ALIGN_X", "AC1_CHIP_ALIGN_Y",
    "AC1_CHIP_ALIGN_T", "AC1_TEST_BIN", "AC1_ARM2_COLLET", "AC1_ALARM",
    "AC2_ARM1_COLLET", "AC2_STAGE_COUNT", "AC2_PROBE_COUNT",
    "AC2_PROBE_X1", "AC2_PROBE_Y1", "AC2_PROBE_X2", "AC2_PROBE_Y2",
    "AC2_STAGE_Z", "AC2_PIN_Z", "AC2_CHIP_ALIGN_X", "AC2_CHIP_ALIGN_Y",
    "AC2_CHIP_ALIGN_T", "AC2_TEST_BIN", "AC2_ARM2_COLLET", "AC2_ALARM",
    "DC2_ARM1_COLLET", "DC2_STAGE_COUNT", "DC2_PROBE_COUNT",
    "DC2_PROBE_X1", "DC2_PROBE_Y1", "DC2_PROBE_X2", "DC2_PROBE_Y2",
    "DC2_STAGE_Z", "DC2_PIN_Z", "DC2_CHIP_ALIGN_X", "DC2_CHIP_ALIGN_Y",
    "DC2_CHIP_ALIGN_T", "DC2_TEST_BIN", "DC2_ARM2_COLLET", "DC2_ALARM",
    "IP_ARM1_COLLET", "IP_STAGE_COUNT", "IP_SURF_BIN", "IP_ARM2_COLLET",
    "IP_BACK_BIN", "IP_ALARM",
    "ULD_PRE_ALIGN_X", "ULD_PRE_ALIGN_Y", "ULD_PRE_ALIGN_T",
    "ULD_POCKET_X", "ULD_POCKET_Y", "ULD_POCKET_ALIGN_X", "ULD_POCKET_ALIGN_Y",
    "ULD_ARM1_COLLET", "ULD_CHIP_ALIGN_X", "ULD_CHIP_ALIGN_Y",
    "ULD_CHIP_ALIGN_NUM", "ULD_ALARM",
];

// TIMESTAMP型のカラムのリスト
const TIMESTAMP_COLUMNS: &[&str] = &["LD_PICKUP_DATE", "ULD_PUT_DATE"];

// カラム名が安全かどうかチェック
fn validate_column_name(column: &str) -> Result<String, String> {
    let upper_column = column.to_uppercase();
    if ALLOWED_COLUMNS.contains(&upper_column.as_str()) {
        Ok(upper_column)
    } else {
        Err(format!("Invalid column name: {}", column))
    }
}

// カラムの型に応じたキャストを取得
fn get_column_cast(column: &str) -> &str {
    let upper_column = column.to_uppercase();
    if INTEGER_COLUMNS.contains(&upper_column.as_str()) {
        "::integer"
    } else if TIMESTAMP_COLUMNS.contains(&upper_column.as_str()) {
        "::timestamp"
    } else {
        "" // VARCHAR型などはキャスト不要
    }
}

// 比較演算子が安全かどうかチェック
fn validate_comparison(comparison: &str) -> Result<String, String> {
    let upper_comparison = comparison.to_uppercase();
    if ALLOWED_COMPARISONS.contains(&upper_comparison.as_str()) {
        Ok(upper_comparison)
    } else {
        Err(format!("Invalid comparison operator: {}", comparison))
    }
}

// グラフ条件から適切なSQL文を作成（パラメータ化バージョン）
// 戻り値: (SQL文, バインドするパラメータのベクタ)
pub fn create_sql(graph_condition: &GraphCondition) -> Result<(String, Vec<String>), String> {
    let mut sql = String::from("SELECT ");
    let mut params: Vec<String> = Vec::new();

    // カラム名のバリデーション
    let x_item = validate_column_name(&graph_condition.graph_x_item)?;
    let y_item = validate_column_name(&graph_condition.graph_y_item)?;
    let plot_unit = if graph_condition.plot_unit != "None" {
        Some(validate_column_name(&graph_condition.plot_unit)?)
    } else {
        None
    };

    // X, Yデータ取得
    if graph_condition.graph_type=="ScatterPlot"{
        println!("{:?}",graph_condition.alarm.codes);
        //プロット単位をまとめるかどうかで決める
        if graph_condition.alarm.codes.is_empty(){ //アラームプロットを重ねない場合
            if let Some(ref unit) = plot_unit {
                sql += &format!("{}, {}, {} FROM chipdata", unit, x_item, y_item);
            } else {
                sql += &format!("{}, {} FROM chipdata", x_item, y_item);
            }
        }else{
            let alarm_column = validate_column_name(&format!("{}_ALARM", graph_condition.alarm.unit))?;
            if let Some(ref unit) = plot_unit {
                sql += &format!("{}, {}, {}, {} FROM chipdata", unit, x_item, y_item, alarm_column);
            } else {
                sql += &format!("{}, {}, {} FROM chipdata", x_item, y_item, alarm_column);
            }
        }
    }else if  graph_condition.graph_type=="DensityPlot"{
        if let Some(ref unit) = plot_unit {
            sql += &format!("{}, {}, {} FROM chipdata", unit, x_item, y_item);
        } else {
            sql += &format!("{}, {} FROM chipdata", x_item, y_item);
        }
    }else if graph_condition.graph_type=="Histogram"{
        //プロット単位をまとめるかどうかで決める
        if let Some(ref unit) = plot_unit {
            sql += &format!("{}, {} FROM chipdata", unit, x_item);
        } else {
            sql += &format!("{} FROM chipdata", x_item);
        }
    }else if graph_condition.graph_type=="LinePlot"{ //時系列プロットの場合はx軸は必ずLD_PICKUP_DATEをとり、アラームが設定されていればそれも取る
        if graph_condition.alarm.codes.is_empty(){ //アラームプロットを重ねない場合
            if let Some(ref unit) = plot_unit {
                sql += &format!("{}, LD_PICKUP_DATE, {} FROM chipdata", unit, y_item);
            } else {
                sql += &format!("LD_PICKUP_DATE, {} FROM chipdata", y_item);
            }
        }else{//アラームプロットを重ねる場合
            let alarm_column = validate_column_name(&format!("{}_ALARM", graph_condition.alarm.unit))?;
            if let Some(ref unit) = plot_unit {
                sql += &format!("{}, LD_PICKUP_DATE, {}, {} FROM chipdata", unit, y_item, alarm_column);
            } else {
                sql += &format!("LD_PICKUP_DATE, {}, {} FROM chipdata", y_item, alarm_column);
            }
        }
    }

    // フィルター情報追加
    let mut param_index = 1;
    if !graph_condition.filters.is_empty() {
        sql += " WHERE ";
        for (index, filter) in graph_condition.filters.iter().enumerate() {
            let item = validate_column_name(&filter.item)?;
            let comparison = validate_comparison(&filter.comparison)?;
            let cast = get_column_cast(&item);

            if comparison == "LIKE" {
                sql += &format!("{} LIKE ${}", item, param_index);
                params.push(format!("%{}%", filter.value));
            } else {
                sql += &format!("{} {} ${}{}", item, comparison, param_index, cast);
                params.push(filter.value.clone());
            }
            param_index += 1;

            if index + 1 < graph_condition.filters.len() {
                // filter_conjunction も検証
                let conjunction = graph_condition.filter_conjunction.to_uppercase();
                if conjunction != "AND" && conjunction != "OR" {
                    return Err("Invalid filter conjunction".to_string());
                }
                sql += &format!(" {} ", conjunction);
            }
        }
        //パーティション情報追加
        sql += &format!(" AND ld_pickup_date BETWEEN ${}::timestamp AND ${}::timestamp", param_index, param_index + 1);
        params.push(graph_condition.start_date.clone());
        params.push(graph_condition.end_date.clone());
    } else {
        //パーティション情報追加
        sql += &format!(" WHERE ld_pickup_date BETWEEN ${}::timestamp AND ${}::timestamp", param_index, param_index + 1);
        params.push(graph_condition.start_date.clone());
        params.push(graph_condition.end_date.clone());
    }

    debug!("Generated SQL: {}", sql);
    debug!("SQL Params: {:?}", params);

    Ok((sql, params))
}

// アラーム条件にあうレコードのみ取得すようなSQL文を作成（パラメータ化バージョン）
pub fn create_alarm_sql(graph_condition: &GraphCondition) -> Result<(String, Vec<String>), String> {
    let mut sql = String::from("SELECT ");
    let mut params: Vec<String> = Vec::new();

    // カラム名のバリデーション
    let x_item = validate_column_name(&graph_condition.graph_x_item)?;
    let y_item = validate_column_name(&graph_condition.graph_y_item)?;
    let plot_unit = if graph_condition.plot_unit != "None" {
        Some(validate_column_name(&graph_condition.plot_unit)?)
    } else {
        None
    };

    // プロットデータ取得用のSQLを定義
    if graph_condition.graph_type == "LinePlot" {
        //LinePlotの場合はx軸は必ずLD_PICKUP_DATE
        if let Some(ref unit) = plot_unit {
            sql += &format!("{}, LD_PICKUP_DATE, {} FROM chipdata", unit, y_item);
        } else {
            sql += &format!("LD_PICKUP_DATE, {} FROM chipdata", y_item);
        }
    }else if graph_condition.graph_type=="ScatterPlot" || graph_condition.graph_type=="DensityPlot" {
        //プロット単位をまとめるかどうかで決める
        if let Some(ref unit) = plot_unit {
            sql += &format!("{}, {}, {} FROM chipdata", unit, x_item, y_item);
        } else {
            sql += &format!("{}, {} FROM chipdata", x_item, y_item);
        }
    }else if graph_condition.graph_type=="Histogram"{
        //プロット単位をまとめるかどうかで決める
        if let Some(ref unit) = plot_unit {
            sql += &format!("{}, {} FROM chipdata", unit, x_item);
        } else {
            sql += &format!("{} FROM chipdata", x_item);
        }
    }

    //アラームフィルター追加
    sql += " WHERE ";
    let alarm_column = validate_column_name(&format!("{}_ALARM", graph_condition.alarm.unit))?;

    let mut param_index = 1;
    // 複数のアラームコードがある場合はOR条件で結合
    if !graph_condition.alarm.codes.is_empty() {
        if graph_condition.alarm.codes.len() == 1 {
            sql += &format!("{} = ${}::integer", alarm_column, param_index);
            params.push(graph_condition.alarm.codes[0].to_string());
            param_index += 1;
        } else {
            sql += "(";
            for (idx, alarm_code) in graph_condition.alarm.codes.iter().enumerate() {
                sql += &format!("{} = ${}::integer", alarm_column, param_index);
                params.push(alarm_code.to_string());
                param_index += 1;

                if idx + 1 < graph_condition.alarm.codes.len() {
                    sql += " OR ";
                }
            }
            sql += ")";
        }
    }

    // フィルター情報追加
    if !graph_condition.filters.is_empty() {
        sql += " AND ";
        for (index, filter) in graph_condition.filters.iter().enumerate() {
            let item = validate_column_name(&filter.item)?;
            let comparison = validate_comparison(&filter.comparison)?;
            let cast = get_column_cast(&item);

            if comparison == "LIKE" {
                sql += &format!("{} LIKE ${}", item, param_index);
                params.push(format!("%{}%", filter.value));
            } else {
                sql += &format!("{} {} ${}{}", item, comparison, param_index, cast);
                params.push(filter.value.clone());
            }
            param_index += 1;

            if index + 1 < graph_condition.filters.len() {
                // filter_conjunction も検証
                let conjunction = graph_condition.filter_conjunction.to_uppercase();
                if conjunction != "AND" && conjunction != "OR" {
                    return Err("Invalid filter conjunction".to_string());
                }
                sql += &format!(" {} ", conjunction);
            }
        }
    }

    //パーティション情報追加
    sql += &format!(" AND ld_pickup_date BETWEEN ${}::timestamp AND ${}::timestamp", param_index, param_index + 1);
    params.push(graph_condition.start_date.clone());
    params.push(graph_condition.end_date.clone());

    debug!("Generated Alarm SQL: {}", sql);
    debug!("Alarm SQL Params: {:?}", params);

    Ok((sql, params))
}
