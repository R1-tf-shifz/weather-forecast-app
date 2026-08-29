use std::collections::HashMap;

use futures::{StreamExt, stream};
use reqwest::Client;

use crate::services::{ForecastRequest, ForecastService, WeatherForecast, WeatherForecastService};

pub struct Orchestrator {
    services: HashMap<ForecastService, Box<dyn WeatherForecastService>>,
    client: Client,
}

impl Orchestrator {
    pub fn new() -> Self {
        let client = Client::new();
        let services = HashMap::new();
        Self { client, services }
    }

    pub fn add_service(&mut self, service: Box<dyn WeatherForecastService>) {
        let name = service.which_service();
        self.services.insert(name, service);
    }

    pub fn change_key(&mut self, service: ForecastService, key: String) {
        let Some(service) = self.services.get_mut(&service) else {
            return;
        };
        service.change_api_key(key);
    }

    async fn get_forecast(
        &self,
        services: Vec<ForecastService>,
        request: ForecastRequest,
    ) -> HashMap<ForecastService, Option<WeatherForecast>> {
        let available_services = &self.services;

        stream::iter(services)
            .map(|service| {
                let req_clone = request.clone();
                async move {
                    if let Some(current_service) = available_services.get(&service) {
                        let forecast = current_service.forecast(req_clone).await;
                        Some((service, forecast))
                    } else {
                        None
                    }
                }
            })
            .buffer_unordered(10)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .flatten()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
