use super::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://api.open-meteo.com/v1/forecast";
const HOUR_FORMAT_PARAMETER: &str = "temperature_2m";
const DEFAULT_TIME_FORMAT: &str = "unixtime";

#[derive(Serialize, Deserialize)]
struct ForecastParamaters {
    latitude: f32,
    longitude: f32,
    forecast_days: i32,
    temperature_unit: String,
    wind_speed_unit: String,
    hourly: String,
    timeformat: String,
}

impl ForecastParamaters {
    fn from_forecast_request(request: ForecastRequest) -> Self {
        let temperature_unit = request
            .temperature_unit
            .unwrap_or(TemperatureUnit::Celsius)
            .to_string();
        let wind_speed_unit = request.wind_speed_unit.unwrap_or(WindUnit::Ms).to_string();
        Self {
            latitude: request.location.latitude,
            longitude: request.location.longitude,
            forecast_days: i32::from(request.forecast_days),
            hourly: String::from(HOUR_FORMAT_PARAMETER),
            timeformat: String::from(DEFAULT_TIME_FORMAT),
            temperature_unit,
            wind_speed_unit,
        }
    }
}

#[derive(Deserialize, Debug)]
struct OpenMeteoResponse {
    latitude: f32,
    longitude: f32,
    generationtime_ms: f32,
    utc_offset_seconds: i32,
    timezone: String,
    timezone_abbreviation: String,
    elevation: f32,
    hourly_units: HourlyUnits,
    hourly: Hourly,
}

#[derive(Deserialize, Debug)]
struct HourlyUnits {
    time: String,
    temperature_2m: String,
}

#[derive(Deserialize, Debug)]
struct Hourly {
    time: Vec<u64>, //unix timestamp
    temperature_2m: Vec<f32>,
}

pub struct OpenMeteoService<'a> {
    pub client: &'a Client,
}

impl<'a> OpenMeteoService<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    async fn send_request(&self, forecast_request: ForecastRequest) -> Option<reqwest::Response> {
        let forecast_parameters = ForecastParamaters::from_forecast_request(forecast_request);
        self.client
            .get(BASE_URL)
            .query(&forecast_parameters)
            .send()
            .await
            .ok()
    }
}

impl<'a> WeatherForecastService for OpenMeteoService<'a> {
    async fn forecast(&self, request: ForecastRequest) -> Option<WeatherForecast> {
        let api_response = self.send_request(request).await?;
        let json_response = api_response.json::<OpenMeteoResponse>().await.ok()?;
        let mut result = WeatherForecast::new();
        for (id, time) in json_response.hourly.time.into_iter().enumerate() {
            let point = WeatherPoint::new(time, json_response.hourly.temperature_2m[id]);
            result.push(point);
        }
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_final_url() {
        let test_url = "https://api.open-meteo.com/v1/forecast?latitude=52.52&longitude=13.41&forecast_days=1&temperature_unit=celsius&wind_speed_unit=ms&hourly=temperature_2m&timeformat=unixtime";
        let location = Location::new(52.52, 13.41);
        let parameters = ForecastRequest::new(location, 1, None, None, None);
        let parameters = ForecastParamaters::from_forecast_request(parameters);
        let client = Client::new();
        let final_url = client.get(BASE_URL).query(&parameters).build().unwrap();
        assert_eq!(test_url, final_url.url().to_string());
    }

    #[tokio::test]
    #[ignore]
    async fn is_api_works() {
        let location = Location::new(52.52, 13.41);
        let parameters = ForecastRequest::new(location, 1, None, None, None);
        let client = Client::new();
        let open_meteo_service = OpenMeteoService::new(&client);
        let result = open_meteo_service.forecast(parameters).await;
        assert!(result.is_some());
    }
}
