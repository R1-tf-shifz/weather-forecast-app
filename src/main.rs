use std::time::Duration;

use reqwest::Client;

mod services;

fn main() {
    let mut builder = Client::builder();
    builder = builder.timeout(Duration::new(1, 0));
    let Ok(client) = builder.build() else {
        return;
    };
    client.get("rl");
}
