use super::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
const BASE_URL: &str = "https://api.open-meteo.com/v1/forecast";

#[derive(Serialize, Deserialize)]
struct ForecastParamaters {
    latitude: f32,
    longitude: f32,
    forecast_days: i32,
    temperature_unit: TemperatureUnit,
    wind_speed_unit: WindUnit,
}

impl ForecastParamaters {
    fn from_forecast_request(request: ForecastRequest) -> Self {
        let temperature_unit = request.temperature_unit.unwrap_or(TemperatureUnit::Celsius);
        let wind_speed_unit = request.wind_speed_unit.unwrap_or(WindUnit::Ms);
        Self {
            latitude: request.location.latitude,
            longitude: request.location.longitude,
            forecast_days: i32::from(request.forecast_days),
            temperature_unit,
            wind_speed_unit,
        }
    }
}

pub struct OpenMeteoService<'a> {
    pub client: &'a Client,
}

impl<'a> OpenMeteoService<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }
}

impl<'a> WeatherForecastService for OpenMeteoService<'a> {
    fn forecast(request: ForecastRequest) -> Option<WeatherForecast> {
        let forecast_parameters = ForecastParamaters::from_forecast_request(request);

        None
    }
}
