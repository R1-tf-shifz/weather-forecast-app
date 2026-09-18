use std::collections::HashMap;

use futures::{StreamExt, stream};
use reqwest::Client;
use thiserror::Error;

use crate::services::{
    ForecastRequest, ForecastService, Location, WeatherForecast, WeatherForecastService,
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
    #[error("Failed to update location")]
    UpdateFailed,
}

pub struct Orchestrator {
    services: HashMap<ForecastService, Box<dyn WeatherForecastService>>,
    request: ForecastRequest,
    client: Client,
}

impl Default for Orchestrator {
    fn default() -> Self {
        let request = ForecastRequest::default();
        Self::new(request)
    }
}

impl Orchestrator {
    pub fn new(request: ForecastRequest) -> Self {
        let client = Client::new();
        let services = HashMap::new();
        Self {
            client,
            services,
            request,
        }
    }

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

    pub async fn single_forecast(&self, service: &ForecastService) -> Option<WeatherForecast> {
        let service = self.services.get(service)?;
        service.forecast(&self.client, self.request.clone()).await
    }

    pub async fn multiple_forecast(
        &self,
        services: Vec<ForecastService>,
    ) -> HashMap<ForecastService, Option<WeatherForecast>> {
        let available_services = &self.services;
        let client = &self.client;

        stream::iter(services)
            .map(|service| {
                let req_clone = self.request.clone();
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

    pub async fn all_forecasts(&self) -> HashMap<ForecastService, Option<WeatherForecast>> {
        stream::iter(&self.services)
            .map(|service| {
                let req = self.request.clone();
                async move {
                    let forecast = service.1.forecast(&self.client, req).await;
                    Some((service.0.clone(), forecast))
                }
            })
            .buffer_unordered(10)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .flatten()
            .collect()
    }

    pub fn update_request(&mut self, request: ForecastRequest) {
        self.request = request;
    }

    pub fn get_request(&self) -> ForecastRequest {
        self.request.clone()
    }

    pub async fn update_location_ip_api(&mut self) -> Result<(), OrchestratorError> {
        self.request.location = match Location::ip_api_location(&self.client).await {
            Some(loc) => loc,
            None => return Err(OrchestratorError::UpdateFailed),
        };

        Ok(())
    }

    pub fn update_location(&mut self, location: Location) {
        self.request.location = location;
    }
}

#[cfg(test)]
mod tests {
    use crate::services::Location;

    use super::*;
    use std::env;

    fn create_orchestrator_with_service(api_key: String) -> Orchestrator {
        let location = Location::new(55.7558, 37.6173);
        let request = ForecastRequest::new(location, 1, None, None);
        let mut orch = Orchestrator::new(request);
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
        let forecast = orch.multiple_forecast(services).await;
        println!("forecast: {:?}", forecast);
        let final_forecast = forecast.get(&ForecastService::VisualCrossing);
        assert!(final_forecast.is_some())
    }

    #[ignore]
    #[tokio::test]
    async fn update_location_ip_api() {
        let mut orch = create_orchestrator_with_service("key".to_string());
        let res = orch.update_location_ip_api().await;
        assert!(res.ok().is_some())
    }
}
