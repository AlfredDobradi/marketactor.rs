use std::ops::RangeInclusive;
use actors::actor;

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set global subscriber");

    let manager = actor::Manager::build(5);

    let mut set = tokio::task::JoinSet::new();
    for type_id in requests() {
        let mut handle = manager.clone();
        set.spawn(async move {
            handle.send_history_request(10000002, type_id).await;
        });
    }

    set.join_all().await;

    loop{}
}

fn requests() -> Vec<i64> {
    let range: RangeInclusive<i64> = 28808..=28813;
    range.collect()
}
