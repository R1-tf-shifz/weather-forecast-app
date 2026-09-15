use super::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const BASE_URL: &str =
    "https://weather.visualcrossing.com/VisualCrossingWebServices/rest/services/timeline";
const DEFAULT_INCLUDE: &str = "current,hours";

#[derive(Serialize, Deserialize)]
struct ForecastParameters {
    key: String,
    #[serde(rename = "unitGroup")]
    unit_group: String,
    include: String,
}

impl ForecastParameters {
    fn from_forecast_request(request: ForecastRequest, key: String) -> Self {
        let unit_group = match request.temperature_unit.unwrap_or_default() {
            TemperatureUnit::Celsius => "metric",
            TemperatureUnit::Fahrenheit => "us",
        };
        ForecastParameters {
            key,
            unit_group: unit_group.to_string(),
            include: DEFAULT_INCLUDE.to_string(),
        }
    }
}

#[derive(Deserialize, Serialize)]
struct VisualCrossingResponse {
    pub days: Vec<Day>,
}

#[derive(Deserialize, Serialize)]
struct Day {
    pub hours: Vec<Hour>,
}

#[derive(Deserialize, Serialize)]
struct Hour {
    #[serde(rename = "datetimeEpoch")]
    pub datetime_epoch: u64,
    pub temp: f32,
    pub feelslike: f32,
}

pub struct VisualCrossingService {
    pub api_key: String,
}

impl VisualCrossingService {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    fn build_url(request: &ForecastRequest) -> String {
        let days = (request.forecast_days - 1).clamp(0, u8::MAX);
        format!(
            "{}/{},{}/next{}days",
            BASE_URL, request.location.latitude, request.location.longitude, days
        )
    }

    async fn send_request(
        &self,
        client: &Client,
        request: ForecastRequest,
    ) -> Option<reqwest::Response> {
        let url = VisualCrossingService::build_url(&request);
        let parameters = ForecastParameters::from_forecast_request(request, self.api_key.clone());
        client.get(url).query(&parameters).send().await.ok()
    }
}

#[async_trait::async_trait]
impl WeatherForecastService for VisualCrossingService {
    async fn forecast(&self, client: &Client, request: ForecastRequest) -> Option<WeatherForecast> {
        let api_response = self.send_request(client, request).await?;
        let json = api_response.json::<VisualCrossingResponse>().await.ok()?;
        let mut result = WeatherForecast::new();

        for day in json.days.into_iter() {
            for hour in day.hours.into_iter() {
                let weather_point =
                    WeatherPoint::new(hour.datetime_epoch, hour.temp, hour.feelslike);
                result.push(weather_point);
            }
        }

        Some(result)
    }

    fn which_service(&self) -> ForecastService {
        ForecastService::VisualCrossing
    }

    fn change_api_key(&mut self, key: String) {
        self.api_key = key;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    #[test]
    fn check_base_url_builder() {
        let location = Location::new(55.7558, 37.6173);
        let parameters = ForecastRequest::new(location, 1, None, None);
        let final_url = VisualCrossingService::build_url(&parameters);
        let test = "https://weather.visualcrossing.com/VisualCrossingWebServices/rest/services/timeline/55.7558,37.6173/next0days";
        assert_eq!(final_url, test);
    }

    #[test]
    fn check_final_url() {
        let test_url = "https://weather.visualcrossing.com/VisualCrossingWebServices/rest/services/timeline/55.7558,37.6173/next0days?key=my_api_key&unitGroup=metric&include=current%2Chours";
        let location = Location::new(55.7558, 37.6173);
        let api_key = "my_api_key".to_string();
        let parameters = ForecastRequest::new(location, 1, None, None);
        let url = VisualCrossingService::build_url(&parameters);
        let parameters = ForecastParameters::from_forecast_request(parameters, api_key);
        let client = Client::new();
        let final_url = client
            .get(url)
            .query(&parameters)
            .build()
            .expect("must work");
        assert_eq!(final_url.url().to_string(), test_url);
    }

    #[tokio::test]
    #[ignore]
    async fn is_api_works() {
        let location = Location::new(55.7558, 37.6173);
        let api_key = env::var("VISUAL_CROSSING_API_KEY").expect("key must be in env");
        let request = ForecastRequest::new(location, 1, None, None);
        let client = Client::new();
        let visual_crossing_service = VisualCrossingService::new(api_key);
        let result = visual_crossing_service.forecast(&client, request).await;
        assert!(result.is_some());
    }
}
