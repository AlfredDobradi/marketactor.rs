use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct HistoryItem {
    pub average: f64,
    pub date: String,
    pub highest: f64,
    pub lowest: f64,
    pub order_count: i64,
    pub volume: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AggregatedItem {
    pub type_id: i64,
    pub average: f64,
}

// pub async fn get_aggregated_item_history(region_id: i64, type_id: i64) -> Result<AggregatedItem, reqwest::Error> {


//         Ok(AggregatedItem {
//             type_id,
//             average: avg,
//         })
// }