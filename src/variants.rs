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

#[derive(Debug,Serialize,Clone)]
pub struct AlarmCounts {
    pub ld_alarm: BTreeMap<i32, u32>,
    pub dc1_alarm: BTreeMap<i32, u32>,
    pub ac1_alarm: BTreeMap<i32, u32>,
    pub ac2_alarm: BTreeMap<i32, u32>,
    pub dc2_alarm: BTreeMap<i32, u32>,
    pub ip_alarm: BTreeMap<i32, u32>,
    pub uld_alarm: BTreeMap<i32, u32>,
}

impl AlarmCounts{
    pub fn new(&mut self,ld_vec:Vec<i32>,dc1_vec:Vec<i32>,ac1_vec:Vec<i32>,ac2_vec:Vec<i32>,dc2_vec:Vec<i32>,ip_vec:Vec<i32>,uld_vec:Vec<i32>){
        //ld部分作成
        for key in ld_vec{
            self.ld_alarm.entry(key).or_insert(0);
        }
        //dc1部分作成
        for key in dc1_vec{
            self.dc1_alarm.entry(key).or_insert(0);
        }
        //ac1部分作成
        for key in ac1_vec{
            self.ac1_alarm.entry(key).or_insert(0);
        }
        //ac2部分作成
        for key in ac2_vec{
            self.ac2_alarm.entry(key).or_insert(0);
        }
        //dc2部分作成
        for key in dc2_vec{
            self.dc2_alarm.entry(key).or_insert(0);
        }
        //ip部分作成
        for key in ip_vec{
            self.ip_alarm.entry(key).or_insert(0);
        }
        //uld部分作成
        for key in uld_vec{
            self.uld_alarm.entry(key).or_insert(0);
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
    pub alarm_counts: AlarmCounts,
}

impl LotUnitData {
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
