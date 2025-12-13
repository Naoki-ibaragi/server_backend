use sqlx::{PgPool, Row,Column};
use sqlx::postgres::PgRow;
use std::collections::{BTreeMap, HashMap};
use serde_json;
use std::{fs};
use std::error::Error;
use chrono::NaiveDateTime;
use tracing::debug;

use crate::variants::{ChipRecord,AlarmDetail,LotUnitData,AlarmCounts};

pub async fn get_alarmdata(pool: &PgPool, alarm_json_path:&str,machine_id:i32,start_date:&str,end_date:&str) -> Result<HashMap<String, LotUnitData>,Box<dyn Error>> {

    //アラームコード一覧をjsonから読み込み
    let s=fs::read_to_string(alarm_json_path)?;
    let alarm_detail:AlarmDetail = serde_json::from_str(&s)?;
    debug!("Loaded alarm detail from JSON: {:#?}", alarm_detail);

    //alarm_detailから各ユニット毎のキー一覧を追加する
    let mut ld_alarmcode_vec:Vec<i32>=vec![];
    for x in alarm_detail.ld_alarm.keys(){
        ld_alarmcode_vec.push(x.parse::<i32>()?)
    }

    let mut dc1_alarmcode_vec:Vec<i32>=vec![];
    for x in alarm_detail.dc1_alarm.keys(){
        dc1_alarmcode_vec.push(x.parse::<i32>()?)
    }

    let mut ac1_alarmcode_vec:Vec<i32>=vec![];
    for x in alarm_detail.ac1_alarm.keys(){
        ac1_alarmcode_vec.push(x.parse::<i32>()?)
    }

    let mut ac2_alarmcode_vec:Vec<i32>=vec![];
    for x in alarm_detail.ac2_alarm.keys(){
        ac2_alarmcode_vec.push(x.parse::<i32>()?)
    }

    let mut dc2_alarmcode_vec:Vec<i32>=vec![];
    for x in alarm_detail.dc2_alarm.keys(){
        dc2_alarmcode_vec.push(x.parse::<i32>()?)
    }

    let mut ip_alarmcode_vec:Vec<i32>=vec![];
    for x in alarm_detail.ip_alarm.keys(){
        ip_alarmcode_vec.push(x.parse::<i32>()?)
    }

    let mut uld_alarmcode_vec:Vec<i32>=vec![];
    for x in alarm_detail.uld_alarm.keys(){
        uld_alarmcode_vec.push(x.parse::<i32>()?)
    }

    //countが全て0の初期状態のAlarmCountsを作成する
    let mut alarm_count_base=AlarmCounts{
        ld_alarm:BTreeMap::new(),
        dc1_alarm:BTreeMap::new(),
        ac1_alarm:BTreeMap::new(),
        ac2_alarm:BTreeMap::new(),
        dc2_alarm:BTreeMap::new(),
        ip_alarm:BTreeMap::new(),
        uld_alarm:BTreeMap::new(),
    };

    AlarmCounts::new(
        &mut alarm_count_base,
        ld_alarmcode_vec,
        dc1_alarmcode_vec,
        ac1_alarmcode_vec,
        ac2_alarmcode_vec,
        dc2_alarmcode_vec,
        ip_alarmcode_vec,
        uld_alarmcode_vec,
    );

    // プールから接続を使用

    // 日付文字列をNaiveDateTimeに変換
    let start_dt = NaiveDateTime::parse_from_str(start_date, "%Y-%m-%d %H:%M:%S")?;
    let end_dt = NaiveDateTime::parse_from_str(end_date, "%Y-%m-%d %H:%M:%S")?;

    // アラームデータを取得
    let sql = "SELECT type_name, lot_name, ld_alarm, dc1_alarm, ac1_alarm, ac2_alarm, dc2_alarm, ip_alarm, uld_alarm
               FROM CHIPDATA
               WHERE machine_id = $1 AND ld_pickup_date BETWEEN $2 AND $3
               ORDER BY serial ASC";

    let rows = sqlx::query(sql)
        .bind(machine_id)
        .bind(start_dt)
        .bind(end_dt)
        .fetch_all(pool)
        .await?;

    // ロット名一覧を取得してlotdateテーブルから日付情報を取得
    let mut lot_dates: HashMap<String, (String, String)> = HashMap::new();

    // lot_name をキーに LotUnitData を格納
    let mut all_lots_hashmap: HashMap<String, LotUnitData> = HashMap::new();

    for row in rows {
        let type_name: Option<String> = row.try_get(0)?;
        let lot_name: Option<String> = row.try_get(1)?;
        let ld_alarm: Option<i32> = row.try_get(2)?;
        let dc1_alarm: Option<i32> = row.try_get(3)?;
        let ac1_alarm: Option<i32> = row.try_get(4)?;
        let ac2_alarm: Option<i32> = row.try_get(5)?;
        let dc2_alarm: Option<i32> = row.try_get(6)?;
        let ip_alarm: Option<i32> = row.try_get(7)?;
        let uld_alarm: Option<i32> = row.try_get(8)?;

        let lot_name = match lot_name {
            Some(s) => s,
            None => continue
        };

        let type_name = match type_name {
            Some(s) => s,
            None => continue
        };

        // lot_start_timeとlot_end_timeをキャッシュから取得、なければDBから取得
        if !lot_dates.contains_key(&lot_name) {
            let sql = "SELECT start_date, end_date FROM lotdate WHERE lot_name = $1";
            let metadata = sqlx::query(sql)
                .bind(&lot_name)
                .fetch_one(pool)
                .await?;

            let start_date: NaiveDateTime = metadata.try_get("start_date")?;
            let end_date: NaiveDateTime = metadata.try_get("end_date")?;

            lot_dates.insert(
                lot_name.clone(),
                (start_date.to_string(), end_date.to_string())
            );
        }

        let (lot_start_time, lot_end_time) = lot_dates.get(&lot_name).unwrap();

        // HashMapにキーが無ければ新規作成
        let lot_entry = all_lots_hashmap
            .entry(lot_name.clone())
            .or_insert_with(|| {
                LotUnitData {
                    machine_id: machine_id,
                    type_name: type_name.clone(),
                    lot_start_time: lot_start_time.clone(),
                    lot_end_time: lot_end_time.clone(),
                    alarm_counts: alarm_count_base.clone()
                }
            });

        // 各アラームをカウント
        if let Some(code) = ld_alarm {
            if let Some(count) = lot_entry.alarm_counts.ld_alarm.get_mut(&code) {
                *count += 1;
            }
        }

        if let Some(code) = dc1_alarm {
            if let Some(count) = lot_entry.alarm_counts.dc1_alarm.get_mut(&code) {
                *count += 1;
            }
        }

        if let Some(code) = ac1_alarm {
            if let Some(count) = lot_entry.alarm_counts.ac1_alarm.get_mut(&code) {
                *count += 1;
            }
        }

        if let Some(code) = ac2_alarm {
            if let Some(count) = lot_entry.alarm_counts.ac2_alarm.get_mut(&code) {
                *count += 1;
            }
        }

        if let Some(code) = dc2_alarm {
            if let Some(count) = lot_entry.alarm_counts.dc2_alarm.get_mut(&code) {
                *count += 1;
            }
        }

        if let Some(code) = ip_alarm {
            if let Some(count) = lot_entry.alarm_counts.ip_alarm.get_mut(&code) {
                *count += 1;
            }
        }

        if let Some(code) = uld_alarm {
            if let Some(count) = lot_entry.alarm_counts.uld_alarm.get_mut(&code) {
                *count += 1;
            }
        }
    }

    // プールは自動的に管理されるため、closeは不要

    Ok(all_lots_hashmap)
}
