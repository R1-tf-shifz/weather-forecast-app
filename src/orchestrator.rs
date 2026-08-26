use std::collections::HashMap;

use reqwest::Client;

use crate::services::{
    ForecastService, WeatherForecastService, open_meteo_service::OpenMeteoService,
};

pub struct Orchestrator<'a> {
    services: HashMap<String, ForecastService<'a>>,
    client: Client,
}

impl<'a> Orchestrator<'a> {
    pub fn new(client: Client) -> Self {
        let services: HashMap<String, ForecastService<'a>> = HashMap::new();
        let services = HashMap::new();
        Self { client, services }
    }

    pub fn initialize(&'a mut self) {
        let open_meteo_service = OpenMeteoService::new(&self.client);
        let service = ForecastService::OpenMeteo(open_meteo_service);
        self.services.insert(service.to_string(), service);
    }

    pub fn add_service(&'a mut self, service: ForecastService<'a>) {
        self.services.insert(service.to_string(), service);
    }

    pub fn change_key(&'a mut self, name: String, key: String) {
        let Some(service) = self.services.get_mut(&name) else {
            return;
        };
        service.change_api_key(key);
    }

    fn get_forecast() {}
}

struct Service<T: WeatherForecastService> {
    service: T,
    available: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
}
