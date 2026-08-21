use serde::{Deserialize, Serialize};

mod open_meteo_service;

#[derive(Serialize, Deserialize, Debug)]
pub struct ForecastRequest {
    pub location: Location,
    pub forecast_days: u8,
    pub api_key: Option<String>,
    pub temperature_unit: Option<TemperatureUnit>,
    pub wind_speed_unit: Option<WindUnit>,
}

impl ForecastRequest {
    fn new(
        location: Location,
        forecast_days: u8,
        api_key: Option<String>,
        temperature_unit: Option<TemperatureUnit>,
        wind_speed_unit: Option<WindUnit>,
    ) -> Self {
        Self {
            location,
            forecast_days,
            api_key,
            temperature_unit,
            wind_speed_unit,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WeatherForecast {}

impl WeatherForecast {
    fn new() -> Self {
        Self {}
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Location {
    pub latitude: f32,
    pub longitude: f32,
}

impl Location {
    fn new(latitude: f32, longitude: f32) -> Self {
        Self {
            latitude,
            longitude,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TemperatureUnit {
    Fahrenheit,
    Celsius,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum WindUnit {
    Ms,
    Mph,
}

pub trait WeatherForecastService {
    fn forecast(request: ForecastRequest) -> Option<WeatherForecast>;
}
