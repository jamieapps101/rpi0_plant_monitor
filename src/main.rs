mod sensors;
mod communications;
mod monitor;
mod types;
mod util;

#[tokio::main]
async fn main() {
    monitor::begin("ubuntu2",1883).await;
}
