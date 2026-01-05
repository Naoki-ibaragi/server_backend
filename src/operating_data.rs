use sqlx::{PgPool, Row,Column};
use serde::Serialize;
use chrono::{NaiveDateTime,TimeDelta};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::error::Error;


#[derive(Debug, Serialize)]
struct OperatingSummary{
    lot_name:String,
    type_name:String,
    lot_start_time:NaiveDateTime,
    lot_end_time:NaiveDateTime,
    lot_time:f64, //ロット流動時間(s)
    start_time:f64, //ロット稼働中時間
    stop_time:f64, //ロット停止中時間
    stop_time_map:HashMap<String,f64>, //各要素毎のロット停止時間
    stop_count_map:HashMap<String,u64>, //各要素毎のロット停止回数
    alarm_detail_map:HashMap<String,String>
}

pub async fn get_operating_data(pool: &PgPool,machine_id:i32,start_date:&str,end_date:&str) -> Result<Vec<OperatingSummary>,Box<dyn Error>> {
    //アラーム番号と項目の対応表を定義
    let mut ld_alarm_map:HashMap<u64,String>=HashMap::new();
    let mut dc1_alarm_map:HashMap<u64,String>=HashMap::new();
    let mut ac1_alarm_map:HashMap<u64,String>=HashMap::new();
    let mut ac2_alarm_map:HashMap<u64,String>=HashMap::new();
    let mut dc2_alarm_map:HashMap<u64,String>=HashMap::new();
    let mut ip_alarm_map:HashMap<u64,String>=HashMap::new();
    let mut uld_alarm_map:HashMap<u64,String>=HashMap::new();

    //アラーム番号と項目の対応表を読み込む
    load_alarm_map(&mut ld_alarm_map, &mut dc1_alarm_map, &mut ac1_alarm_map, &mut ac2_alarm_map, &mut dc2_alarm_map, &mut ip_alarm_map, &mut uld_alarm_map);

    //最初にlotdateテーブルからロットのstart_date~end_date内の該当設備のロット番号一覧を取得する
    let sql="SELECT lot_name FROM lotdate WHERE machine_id = $1 and start_date > $2 and end_date < $3 ORDER BY start_date ASC";
    let rows = sqlx::query(sql).bind(machine_id).bind(start_date).bind(end_date)
    .fetch_all(pool).await?;

    let mut summary_vec:Vec<OperatingSummary> = Vec::new();

    //eventsテーブルから各ロットの稼働データを取得する
    for row in rows{
        let lot_name=row.get("lot_name");

        let event_sql="SELECT type_name,date,event_type,alarm_unit,alarm_code FROM events WHERE lot_name= $1 ORDER BY date ASC";
        let event_rows = sqlx::query(event_sql).bind(lot_name)
        .fetch_all(pool).await?;

        let mut type_name_global:String = String::new();
        let mut lot_start_flag=false; //ロット開始が記録されているか
        let mut lot_end_flag=false; //ロット終了が記録されているか
        let mut start_state=false; //現在設備稼働状態にあるか
        let mut stop_state=false; //現在設備停止状態にあるか
        let mut alarm_state=false; //現在設備停止状態にあるか
        let mut lot_start_date:NaiveDateTime = NaiveDateTime::default(); //ロット開始時刻
        let mut lot_end_date:NaiveDateTime = NaiveDateTime::default(); //ロット終了時刻
        let mut lot_time:f64=0.; //ロット流動時間
        let mut start_time:f64=0.; //稼働時間
        let mut stop_time:f64=0.; //停止時間
        let mut before_date:NaiveDateTime = NaiveDateTime::default(); //一つ前のイベント時刻
        let mut current_alarm_unit:String = String::new();
        let mut current_alarm_code:i32 = 0;
        let mut stop_time_map:HashMap<String,f64>=HashMap::new(); //アラーム停止時間をアラーム毎に秒単位で加算していく
        let mut stop_count_map:HashMap<String,u64>=HashMap::new(); //アラーム停止回数をアラームコード毎にまとめる
        let mut alarm_detail_map:HashMap<String,String>=HashMap::new(); //アラームコードの詳細


        //稼稼働デーから情報をまとめる
        //1.ロット開始時刻・ロット終了時刻
        //2.ロット流動時間
        //3.アラーム種類ごとの停止時間
        //4.その他停止時間
        for row in event_rows{
            let type_name:String=row.get("type_name");
            let event_type:String=row.get("event_type");
            let date: NaiveDateTime = row.get("date");
            let alarm_unit:String=row.get("alarm_unit");
            let alarm_code:i32=row.get("alarm_code");
           
            if event_type=="LOT_START"{
                //ロット開始時刻を記録
                lot_start_date=date.clone();
                
                //状態状更新
                lot_start_flag=true;
                start_state=true; 
                stop_state=false;
                alarm_state=false;

                //現在の時刻を記録
                before_date=date.clone();

            }else if event_type=="LOT_END"{
                lot_end_date=date.clone();
                lot_end_flag=true;
                type_name_global=type_name.clone();
                if lot_start_flag{
                    lot_time=TimeDelta::as_seconds_f64(lot_end_date-lot_start_date); //ロット流動時間(s)
                }
            }else if event_type=="ALARM" && alarm_state==false{ //2次発生アラームは考慮しない
                if start_state==true{ //稼働状態であればここまでの稼働時間を加算
                    start_time+=TimeDelta::as_seconds_f64(date-before_date);
                    start_state=false;
                }

                //current_alarm_codeとcurrent_alarm_unitを更新する
                current_alarm_code=alarm_code;
                current_alarm_unit=alarm_unit;

                //状態状更新
                stop_state=true;
                alarm_state=true;
                start_state=false;

                //現在の時刻を記録
                before_date=date.clone();
            }else if event_type=="START"{ //再稼働時のイベント
                //停止状態からの再スタートであればここまでの稼働時間を加算
                //さらにアラーム停止中であればstop_time_mapも更新する
                if stop_state==true && alarm_state==false{ 
                    stop_time+=TimeDelta::as_seconds_f64(date-before_date);
                }else if stop_state==true && alarm_state==true{
                    let delta_sec=TimeDelta::as_seconds_f64(date-before_date);
                    stop_time+=delta_sec;

                    //アラームコード毎の停止時間をmapに保存
                    let key=current_alarm_unit.clone()+"_"+&current_alarm_code.to_string();
                    *stop_time_map.entry(key.clone()).or_insert(0.0)+=delta_sec;

                    //アラームコード毎の停止回数をmapに保存
                    *stop_count_map.entry(key.clone()).or_insert(0)+=1;
                    
                    //アラームコードの詳細をmapに保存
                    let detail:String = if current_alarm_unit=="ld"{
                        ld_alarm_map.get(&(current_alarm_code as u64)).unwrap_or(&"".to_string()).to_string()
                    }else if current_alarm_unit=="dc1"{
                        dc1_alarm_map.get(&(current_alarm_code as u64)).unwrap_or(&"".to_string()).to_string()
                    }else if current_alarm_unit=="ac1"{
                        ac1_alarm_map.get(&(current_alarm_code as u64)).unwrap_or(&"".to_string()).to_string()
                    }else if current_alarm_unit=="ac2"{
                        ac2_alarm_map.get(&(current_alarm_code as u64)).unwrap_or(&"".to_string()).to_string()
                    }else if current_alarm_unit=="dc2"{
                        dc2_alarm_map.get(&(current_alarm_code as u64)).unwrap_or(&"".to_string()).to_string()
                    }else if current_alarm_unit=="ip"{
                        ip_alarm_map.get(&(current_alarm_code as u64)).unwrap_or(&"".to_string()).to_string()
                    }else if current_alarm_unit=="uld"{
                        uld_alarm_map.get(&(current_alarm_code as u64)).unwrap_or(&"".to_string()).to_string()
                    }else{
                        "".to_string()
                    };
                    alarm_detail_map.insert(key.clone(), detail);
                }

                //状態状更新
                alarm_state=false;
                stop_state=false;
                start_state=true;

                //現在の時刻を記録
                before_date=date.clone();
            }else if event_type=="LOCK_STOP" || event_type=="NO_LOCK_STOP"{ //stopボタン押し下げ時のイベント
                //稼働状態であればここまでの稼働時間を加算
                if start_state==true{ 
                    let delta_sec=TimeDelta::as_seconds_f64(date-before_date);
                    start_time+=delta_sec;

                    //アラームによる停止でなければ、単純停止時間を追加
                    if alarm_state==false{
                        let key="no_alarm_stop".to_string();
                        *stop_time_map.entry(key).or_insert(delta_sec)+=delta_sec;
                    }
                }

                //状態状更新
                stop_state=true;
                start_state=false;

                //現在の時刻を記録
                before_date=date.clone();
            }
        }

        //lot_start,lot_end共に取得出来ているもののみ結果として取得する
        if lot_start_flag && lot_end_flag{
            summary_vec.push(
                OperatingSummary{
                    lot_name:lot_name,
                    type_name:type_name_global,
                    lot_start_time:lot_start_date,
                    lot_end_time:lot_end_date,
                    lot_time:lot_time,
                    start_time:start_time,
                    stop_time:stop_time,
                    stop_time_map:stop_time_map,
                    stop_count_map:stop_count_map,
                    alarm_detail_map:alarm_detail_map
                }
            )
        }
    }

    Ok(summary_vec)

}

fn load_alarm_map(
    ld_alarm_map:&mut HashMap<u64,String>,
    dc1_alarm_map:&mut HashMap<u64,String>,
    ac1_alarm_map:&mut HashMap<u64,String>,
    ac2_alarm_map:&mut HashMap<u64,String>,
    dc2_alarm_map:&mut HashMap<u64,String>,
    ip_alarm_map:&mut HashMap<u64,String>,
    uld_alarm_map:&mut HashMap<u64,String>,
){
    let path_arr=[
        Path::new("/components/jp_alm_ld.txt"),
        Path::new("/components/jp_alm_dc1.txt"),
        Path::new("/components/jp_alm_ac1.txt"),
        Path::new("/components/jp_alm_ac2.txt"),
        Path::new("/components/jp_alm_dc2.txt"),
        Path::new("/components/jp_alm_ip.txt"),
        Path::new("/components/jp_alm_uld.txt")
    ];

    let map_arr=[
        ld_alarm_map,
        dc1_alarm_map,
        ac1_alarm_map,
        ac2_alarm_map,
        dc2_alarm_map,
        ip_alarm_map,
        uld_alarm_map
    ];

    for i in 0..map_arr.len(){
        let _ = read_alarm_map_data(map_arr[i],path_arr[i]);
    }

}

fn read_alarm_map_data(map:&mut HashMap<u64,String>,path:&Path)->Result<(),Box<dyn Error>>{
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line?; // Result<String> → String
           // 空行スキップ
        if line.trim().is_empty() {
            continue;
        }

        // 最初の空白 or タブで2分割
        let mut parts = line.splitn(2, char::is_whitespace);

        let key: u64 = match parts.next().unwrap().parse() {
            Ok(v) => v,
            Err(_) => continue, // 数字でなければ無視
        };

        let value = parts
            .next()
            .unwrap_or("")
            .trim()
            .to_string();

        map.insert(key, value);
    }

    Ok(())

}
