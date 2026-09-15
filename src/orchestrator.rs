use std::collections::HashMap;

use futures::{StreamExt, stream};
use reqwest::Client;
use thiserror::Error;

use crate::services::{
    ForecastRequest, ForecastService, WeatherForecast, WeatherForecastService,
    open_meteo_service::OpenMeteoService, visual_crossing_service::VisualCrossingService,
};

#[derive(Error, Debug)]
pub enum OrchestratorError {
    #[error("Api key required for creating this service {0}")]
    ApiKeyRequired(ForecastService),
    #[error("Service {0} already created. Value updated")]
    ServiceAlreadyExists(ForecastService),
    #[error("No such service: {0}. You need to add service to update key")]
    NoSuchService(ForecastService),
}

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

    //pub fn add_service(&mut self, service: Box<dyn WeatherForecastService>) {
    //    let name = service.which_service();
    //    self.services.insert(name, service);
    //                Ok(Box::new(VisualCrossingService::new(api_key.unwrap())))
    //            }
    //}

    pub fn add_service(
        &mut self,
        service: ForecastService,
        api_key: Option<String>,
    ) -> Result<(), OrchestratorError> {
        let new_service: Box<dyn WeatherForecastService> = match service {
            ForecastService::OpenMeteo => {
                let service = OpenMeteoService::new();
                Box::new(service) as Box<dyn WeatherForecastService>
            }
            ForecastService::VisualCrossing => {
                if api_key.is_none() {
                    return Err(OrchestratorError::ApiKeyRequired(
                        ForecastService::VisualCrossing,
                    ));
                }
                let service = VisualCrossingService::new(api_key.unwrap());
                Box::new(service) as Box<dyn WeatherForecastService>
            }
        };

        if self.services.insert(service.clone(), new_service).is_some() {
            return Err(OrchestratorError::ServiceAlreadyExists(service));
        }
        Ok(())
    }

    pub fn change_api_key(
        &mut self,
        service: ForecastService,
        key: String,
    ) -> Result<(), OrchestratorError> {
        let Some(service) = self.services.get_mut(&service) else {
            return Err(OrchestratorError::NoSuchService(service));
        };

        service.change_api_key(key);
        Ok(())
    }

    pub async fn get_forecast(
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
    use crate::services::Location;

    use super::*;
    use std::env;

    fn create_orchestrator_with_service(api_key: String) -> Orchestrator {
        let mut orch = Orchestrator::new();
        orch.add_service(ForecastService::VisualCrossing, Some(api_key))
            .expect("must work");
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
        let request = ForecastRequest::new(location, 1, None, None);
        let forecast = orch.get_forecast(services, request).await;
        println!("forecast: {:?}", forecast);
        let final_forecast = forecast.get(&ForecastService::VisualCrossing);
        assert!(final_forecast.is_some())
    }
}
