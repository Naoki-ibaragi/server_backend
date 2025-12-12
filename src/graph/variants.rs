use serde::{Deserialize,Serialize};

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
    pub codes:Vec<i32>,
}
/* ------------------------------------------- */

/*プロットデータ型の定義 */
#[derive(Debug,Serialize)]
pub enum XdimData{
    NumberData(Option<i32>),
    DateData(Option<chrono::NaiveDateTime>)
}

#[derive(Debug,Serialize)]
pub struct ScatterPlotData{
    pub x_data:XdimData, //日付等の文字列と通常数値両方取る可能性がある
    pub y_data:Option<i32>,
    pub is_alarm:bool,
}

#[derive(Debug,Serialize)]
pub struct LinePlotData{
    pub y_data:Option<i32>,
    pub is_alarm:bool,
}

#[derive(Debug,Serialize)]
pub struct HistogramData{
    pub x_data:Option<i32>,
}

#[derive(Debug,Serialize)]
pub struct BinnedHistogramData{
    pub bin_index: usize,     // ビンのインデックス
    pub count: i32,           // そのビンの個数
}

#[derive(Debug,Serialize)]
pub struct HistogramBinInfo{
    pub bin_edges: Vec<i32>,  // ビンの境界値 [min, edge1, edge2, ..., max]
    pub bin_width: f64,       // ビン幅
}

#[derive(Debug,Serialize)]
pub struct HeatmapData{
    pub x_data:u32,
    pub y_data:u32,
    pub z_data:Option<i32>,
}

#[derive(Debug,Serialize)]
pub enum PlotData{
    Scatter(ScatterPlotData),
    Line(LinePlotData),
    Histogram(HistogramData),
    BinnedHistogram(BinnedHistogramData),
    Heatmap(HeatmapData),
}

//ヒートマップ描画でフロントエンド側に返すべき情報
#[derive(Debug,Serialize)]
pub struct GridData{
    pub grid_x:f64,
    pub grid_y:f64,
    pub x_min:i32,
    pub y_min:i32,
    pub histogram_bin_info: Option<HistogramBinInfo>,  // ヒストグラムのビン情報
}

#[derive(Debug,Serialize)]
pub enum UnitType{
    String(String),
    Number(i32)
}

