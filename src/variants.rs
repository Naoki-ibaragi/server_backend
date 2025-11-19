/* 全構造体をここで定義する */
use serde::{Deserialize, Serialize};
use std::collections::{HashMap,BTreeMap};

/* Input Data一覧 */
#[derive(Debug,Deserialize)]
pub struct LotData{
    pub lot_name:String,
}

#[derive(Debug,Deserialize)]
pub struct MachineData{
    pub machine_name:String,
}

/* アラームデータ取得関係の構造体 */
#[derive(Debug)]
pub struct ChipRecord {
    pub machine_name: Option<String>,
    pub type_name: Option<String>,
    pub lot_name: Option<String>,
    pub ld_pickup_date: Option<String>,
    pub uld_put_date: Option<String>,
    pub ld_alarm: Option<i32>,
    pub dc1_alarm: Option<i32>,
    pub ac1_alarm: Option<i32>,
    pub ac2_alarm: Option<i32>,
    pub dc2_alarm: Option<i32>,
    pub ip_alarm: Option<i32>,
    pub uld_alarm: Option<i32>,
}

#[derive(Debug,Serialize)]
pub struct AlarmCounts {
    pub ld_alarm: BTreeMap<i32, u32>,
    pub dc1_alarm: BTreeMap<i32, u32>,
    pub ac1_alarm: BTreeMap<i32, u32>,
    pub ac2_alarm: BTreeMap<i32, u32>,
    pub dc2_alarm: BTreeMap<i32, u32>,
    pub ip_alarm: BTreeMap<i32, u32>,
    pub uld_alarm: BTreeMap<i32, u32>,
}

impl AlarmCounts {
    pub fn new(
        ld_keys:Vec<i32>,
        dc1_keys:Vec<i32>,
        ac1_keys:Vec<i32>,
        ac2_keys:Vec<i32>,
        dc2_keys:Vec<i32>,
        ip_keys:Vec<i32>,
        uld_keys:Vec<i32>,
    ) -> Self {
        let ld_map:BTreeMap<i32,u32>=ld_keys.into_iter().map(|k| (k,0)).collect();
        let dc1_map:BTreeMap<i32,u32>=dc1_keys.into_iter().map(|k| (k,0)).collect();
        let ac1_map:BTreeMap<i32,u32>=ac1_keys.into_iter().map(|k| (k,0)).collect();
        let ac2_map:BTreeMap<i32,u32>=ac2_keys.into_iter().map(|k| (k,0)).collect();
        let dc2_map:BTreeMap<i32,u32>=dc2_keys.into_iter().map(|k| (k,0)).collect();
        let ip_map:BTreeMap<i32,u32>=ip_keys.into_iter().map(|k| (k,0)).collect();
        let uld_map:BTreeMap<i32,u32>=uld_keys.into_iter().map(|k| (k,0)).collect();
        AlarmCounts {
            ld_alarm: ld_map,
            dc1_alarm: dc1_map,
            ac1_alarm: ac1_map,
            ac2_alarm: ac2_map,
            dc2_alarm: dc2_map,
            ip_alarm: ip_map,
            uld_alarm: uld_map,
        }
    }
}

#[derive(Debug, serde::Deserialize,Serialize)]
pub struct AlarmDetail{
    pub ld_alarm:HashMap<String,String>,
    pub dc1_alarm:HashMap<String,String>,
    pub ac1_alarm:HashMap<String,String>,
    pub ac2_alarm:HashMap<String,String>,
    pub dc2_alarm:HashMap<String,String>,
    pub ip_alarm:HashMap<String,String>,
    pub uld_alarm:HashMap<String,String>,
}

#[derive(Debug,Serialize)]
pub struct LotUnitData {
    pub machine_name: String,
    pub type_name: String,
    pub lot_start_time: String,
    pub lot_end_time: String,
    pub alarm_list: AlarmCounts,
}

impl LotUnitData {
    pub fn new(machine_name: &str, 
        type_name: &str, 
        lot_start_time: &str, 
        lot_end_time: &str,
        ld_alarm_vec:Vec<i32>,
        dc1_alarm_vec:Vec<i32>,
        ac1_alarm_vec:Vec<i32>,
        ac2_alarm_vec:Vec<i32>,
        dc2_alarm_vec:Vec<i32>,
        ip_alarm_vec:Vec<i32>,
        uld_alarm_vec:Vec<i32>
        ) -> Self {
        LotUnitData {
            machine_name: machine_name.to_string(),
            type_name: type_name.to_string(),
            lot_start_time: lot_start_time.to_string(),
            lot_end_time: lot_end_time.to_string(),
            alarm_list: AlarmCounts::new(
                ld_alarm_vec.clone(),
                dc1_alarm_vec.clone(),
                ac1_alarm_vec.clone(),
                ac2_alarm_vec.clone(),
                dc2_alarm_vec.clone(),
                ip_alarm_vec.clone(),
                uld_alarm_vec.clone()
            ),
        }
    }

    pub fn check_date(&mut self, ld_time: &str,uld_time:&str) {
        if self.lot_start_time.is_empty(){
            self.lot_start_time=ld_time.to_string();
        }else if self.lot_start_time > ld_time.to_string() {
            self.lot_start_time = ld_time.to_string();
        }

        if self.lot_end_time.is_empty(){
            self.lot_end_time=uld_time.to_string();
        }else if self.lot_end_time < uld_time.to_string() {
            self.lot_end_time = uld_time.to_string();
        }
    }
}
