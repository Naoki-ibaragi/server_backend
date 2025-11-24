use rusqlite::{Connection, Result};
use std::collections::{BTreeMap, HashMap};
use serde_json;
use std::{fs};
use std::error::Error;

use crate::variants::{ChipRecord,AlarmDetail,LotUnitData,AlarmCounts};

pub fn get_alarmdata(db_path: &str, table_json_path:&str,machine_name: &str, alarm_json_path:&str) -> Result<HashMap<String, LotUnitData>,Box<dyn Error>> {

    //アラームコード一覧をjsonから読み込み
    let s=fs::read_to_string(alarm_json_path)?;
    let alarm_detail:AlarmDetail = serde_json::from_str(&s)?;
    println!("alarm_detail:{:#?}",alarm_detail);


    //対象の装置のテーブル名を取得
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

    //dbに接続
    let db = Connection::open(db_path)?;

    let sql=format!("SELECT machine_name,type_name,lot_name,ld_pickup_date,uld_put_date,
    ld_alarm,dc1_alarm,ac1_alarm,ac2_alarm,dc2_alarm,ip_alarm,uld_alarm FROM {table_name}");

    let mut stmt = db.prepare(&sql)?;

    // lot_name をキーに LotUnitData を格納
    //このhasmapを最後にreturnする
    let mut all_lots_hashmap: HashMap<String, LotUnitData> = HashMap::new();

    //テーブル内の全レコードを取得
    let chip_iter:Vec<ChipRecord> = stmt.query_map([], |row| {
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
    })?
    .filter_map(|chip| chip.ok())
    .collect();

    for chip in chip_iter {
        let lot_name = match chip.lot_name.clone(){
            Some(s)=>s.to_string(),
            None=>continue
        };

        let machine_name = match chip.machine_name.clone(){
            Some(s)=>s.to_string(),
            None=>continue
        };

        let type_name = match chip.type_name.clone(){
            Some(s)=>s.to_string(),
            None=>continue
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
        let lot_entry = all_lots_hashmap
            .entry(lot_name.clone())
            .or_insert_with(|| {
                LotUnitData{
                    machine_name:machine_name.clone(),
                    type_name:type_name.clone(),
                    lot_start_time:ld_pickup_date.clone(),
                    lot_end_time:uld_put_date.clone(),
                    alarm_counts:alarm_count_base.clone()
                }
            });

        // lot_start_time / lot_end_time を更新
        lot_entry.check_date(&ld_pickup_date,&uld_put_date);

        //各アラームをカウント
        match chip.ld_alarm{
            Some(s)=>{
                if let Some(v)=lot_entry.alarm_counts.ld_alarm.get_mut(&s){
                    *v+=1;
                };
            },
            None=>{}
        };

        match chip.dc1_alarm{
            Some(s)=>{
                if let Some(v)=lot_entry.alarm_counts.dc1_alarm.get_mut(&s){
                    *v+=1;
                };
            },
            None=>{}
        };

        match chip.ac1_alarm{
            Some(s)=>{
                if let Some(v)=lot_entry.alarm_counts.ac1_alarm.get_mut(&s){
                    *v+=1;
                };
            },
            None=>{}
        };
        match chip.ac2_alarm{
            Some(s)=>{
                if let Some(v)=lot_entry.alarm_counts.ac2_alarm.get_mut(&s){
                    *v+=1;
                };
            },
            None=>{}
        };
        match chip.dc2_alarm{
            Some(s)=>{
                if let Some(v)=lot_entry.alarm_counts.dc2_alarm.get_mut(&s){
                    *v+=1;
                };
            },
            None=>{}
        };
        match chip.ip_alarm{
            Some(s)=>{
                if let Some(v)=lot_entry.alarm_counts.ip_alarm.get_mut(&s){
                    *v+=1;
                };
            },
            None=>{}
        };
        match chip.uld_alarm{
            Some(s)=>{
                if let Some(v)=lot_entry.alarm_counts.uld_alarm.get_mut(&s){
                    *v+=1;
                };
            },
            None=>{}
        };

    }

    Ok(all_lots_hashmap)
}

