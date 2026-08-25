use reqwest::Client;
use std::time::Duration;

mod orchestrator;
mod services;

#[tokio::main]
async fn main() {
    let mut builder = Client::builder();
    builder = builder.timeout(Duration::new(1, 0));
    let Ok(client) = builder.build() else {
        return;
    };
}
