use std::collections::HashMap;

use futures::{StreamExt, stream};
use reqwest::Client;

use crate::services::{ForecastRequest, ForecastService, WeatherForecast, WeatherForecastService};

pub struct Orchestrator {
    services: HashMap<ForecastService, Box<dyn WeatherForecastService>>,
    client: Client,
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
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
        let client = &self.client;

        stream::iter(services)
            .map(|service| {
                let req_clone = request.clone();
                async move {
                    if let Some(current_service) = available_services.get(&service) {
                        let forecast = current_service.forecast(client, req_clone).await;
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
    use crate::services::{Location, visual_crossing_service::VisualCrossingService};

    use super::*;
    use std::env;

    fn create_orchestrator_with_service(api_key: String) -> Orchestrator {
        let mut orch = Orchestrator::new();
        let service = VisualCrossingService::new(api_key);
        let service = Box::new(service);
        orch.add_service(service);
        orch
    }

    #[test]
    fn add_service_test() {
        let orch = create_orchestrator_with_service("test_key".to_string());
        let result = orch.services.get(&ForecastService::VisualCrossing);
        assert!(result.is_some())
    }

    #[tokio::test]
    #[ignore]
    async fn forecast_test() {
        let api_key = env::var("VISUAL_CROSSING_API_KEY").expect("key must be in env");
        let orch = create_orchestrator_with_service(api_key);
        let services = vec![ForecastService::VisualCrossing];
        let location = Location::new(55.7558, 37.6173);
        let request = ForecastRequest::new(location, 1, None, None, None);
        let forecast = orch.get_forecast(services, request).await;
        println!("forecast: {:?}", forecast);
        let final_forecast = forecast.get(&ForecastService::VisualCrossing);
        assert!(final_forecast.is_some())
    }
}
