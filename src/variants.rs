/* 全構造体をここで定義する */
use serde::{Deserialize, Serialize};

/* Input Data一覧 */
#[derive(Debug,Deserialize)]
pub struct LotData{
    pub lot_name:String,
}

#[derive(Debug,Deserialize)]
pub struct MachineData{
    pub machine_name:String,
}

/*グラフ作成条件*/
#[derive(Debug,Deserialize)]
pub struct GraphCondition{ //グラフ描画に必要な情報を全て入れる構造体
    pub graph_type:String,          //グラフ種類
    pub graph_x_item:String,        //x軸の項目
    pub graph_y_item:String,        //y軸の項目
    pub start_date:String,          //データ取得開始日
    pub end_date:String,            //データ取得終了日
    pub bin_number:u32,          //ヒストグラムのX軸の分割数
    pub bins_x:u32,              //密度プロットのX軸の分割数
    pub bins_y:u32,              //密度プロットのY軸軸分割数
    pub plot_unit:String,           //plotの分割設定
    pub alarm:AlarmInfo,            //alarm関係の情報
    pub filters:Vec<Filter>,        //filter一覧
    pub filter_conjunction:String   //filterの接続方法AND or OR
}

#[derive(Debug,Deserialize)]
pub struct Filter{ //各フィルターの内容を入れる構造体
    pub item:String,
    pub value:String,
    pub comparison:String
}

#[derive(Debug,Deserialize)]
pub struct AlarmInfo{ //アラームプロットを重ねる場合：アラームの内容を入れる構造体
    pub unit:String,
    pub codes:Vec<String>,
}
