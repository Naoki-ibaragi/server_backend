use actix_web::cookie::time::error;
use rusqlite::{Connection, Result};
use std::collections::{HashMap};
use std::hash::Hash;
use serde_json;
use std::{fs};
use std::error::Error;

use crate::variants::{ChipRecord,AlarmDetail,LotUnitData};

pub fn get_alarmdata(db_path: &str, table_json_path:&str,machine_name: &str, alarm_json_path:&str) -> Result<(HashMap<String, LotUnitData>,AlarmDetail),Box<dyn Error>> {

    let s=fs::read_to_string(alarm_json_path)?;
    let alarm_detail:AlarmDetail = serde_json::from_str(&s)?;

    let s=fs::read_to_string(table_json_path)?;
    let table_map:HashMap<String,String> = serde_json::from_str(&s)?;
    let table_name=match table_map.get(machine_name){
        Some(name)=>name,
        None=>return Err(format!("{} has no table ",machine_name).into())
    };

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

    let db = Connection::open(db_path)?;

    let sql=format!("SELECT machine_name,type_name,lot_name,ld_pickup_date,uld_put_date,
    ld_alarm,dc1_alarm,ac1_alarm,ac2_alarm,dc2_alarm,ip_alarm,uld_alarm FROM {table_name}");

    let mut stmt = db.prepare(&sql)?;

    // lot_name をキーに LotUnitData を格納
    let mut return_hashmap: HashMap<String, LotUnitData> = HashMap::new();

    let chip_iter = stmt.query_map([], |row| {
        Ok(ChipRecord {
            machine_name: row.get(0)?,
            type_name: row.get(1)?,
            lot_name: row.get(2)?,
            ld_pickup_date: row.get(3)?,
            uld_put_date: row.get(4)?,
            ld_alarm: row.get(5)?,
            dc1_alarm: row.get(6)?,
            ac1_alarm: row.get(7)?,
            ac2_alarm: row.get(8)?,
            dc2_alarm: row.get(9)?,
            ip_alarm: row.get(10)?,
            uld_alarm: row.get(11)?,
        })
    })?;

    for chip in chip_iter {
        let chip = chip?;
        let lot_name = match chip.lot_name.clone(){
            Some(s)=>s.to_string(),
            None=>String::from("")
        };

        let machine_name = match chip.machine_name.clone(){
            Some(s)=>s.to_string(),
            None=>String::from("")
        };

        let type_name = match chip.type_name.clone(){
            Some(s)=>s.to_string(),
            None=>String::from("")
        };

        let ld_pickup_date = match chip.ld_pickup_date.clone(){
            Some(s)=>s.to_string(),
            None=>String::from("")
        };
        let uld_put_date = match chip.uld_put_date.clone(){
            Some(s)=>s.to_string(),
            None=>String::from("")
        };

        // HashMapにキーが無ければ新規作成
        let lot_entry = return_hashmap
            .entry(lot_name.clone())
            .or_insert_with(|| {
                LotUnitData::new(
                    &machine_name,
                    &type_name,
                    &ld_pickup_date,
                    &uld_put_date,
                    ld_alarmcode_vec.clone(),
                    dc1_alarmcode_vec.clone(),
                    ac1_alarmcode_vec.clone(),
                    ac2_alarmcode_vec.clone(),
                    dc2_alarmcode_vec.clone(),
                    ip_alarmcode_vec.clone(),
                    uld_alarmcode_vec.clone(),
                )
            });

        // lot_start_time / lot_end_time を更新
        lot_entry.check_date(&ld_pickup_date,&uld_put_date);

        // 例: 各アラームの非空文字列をカウントに追加（必要に応じて拡張）
        match chip.ld_alarm{
            Some(s)=>{
                *lot_entry
                .alarm_list
                .ld_alarm
                .entry(s.clone())
                .or_insert(0) += 1;
            },
            None=>{}
        }

        match chip.dc1_alarm{
            Some(s)=>{
                *lot_entry
                .alarm_list
                .dc1_alarm
                .entry(s.clone())
                .or_insert(0) += 1;
            },
            None=>{}
        }

        match chip.ac1_alarm{
            Some(s)=>{
                *lot_entry
                .alarm_list
                .ac1_alarm
                .entry(s.clone())
                .or_insert(0) += 1;
            },
            None=>{}
        }

        match chip.ac2_alarm{
            Some(s)=>{
                *lot_entry
                .alarm_list
                .ac2_alarm
                .entry(s.clone())
                .or_insert(0) += 1;
            },
            None=>{}
        }

        match chip.dc2_alarm{
            Some(s)=>{
                *lot_entry
                .alarm_list
                .dc2_alarm
                .entry(s.clone())
                .or_insert(0) += 1;
            },
            None=>{}
        }

        match chip.ip_alarm{
            Some(s)=>{
                *lot_entry
                .alarm_list
                .ip_alarm
                .entry(s.clone())
                .or_insert(0) += 1;
            },
            None=>{}
        }

        match chip.uld_alarm{
            Some(s)=>{
                *lot_entry
                .alarm_list
                .uld_alarm
                .entry(s.clone())
                .or_insert(0) += 1;
            },
            None=>{}
        }
    }

    Ok((return_hashmap,alarm_detail))
}

