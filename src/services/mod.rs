use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fmt;

pub mod open_meteo_service;
pub mod visual_crossing_service;

#[derive(Eq, Hash, PartialEq, Debug, Serialize, Deserialize, Clone)]
pub enum ForecastService {
    VisualCrossing,
    OpenMeteo,
}

impl fmt::Display for ForecastService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result = match self {
            Self::VisualCrossing => "VisualCrossing".to_string(),
            Self::OpenMeteo => "OpenMeteo".to_string(),
        };
        write!(f, "{result}")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ForecastRequest {
    pub location: Location,
    pub forecast_days: u8,
    pub temperature_unit: Option<TemperatureUnit>,
    pub wind_speed_unit: Option<WindUnit>,
}

impl Default for ForecastRequest {
    fn default() -> Self {
        let loc = Location::new(50.0, 50.0);
        ForecastRequest::new(loc, 3, None, None)
    }
}

impl ForecastRequest {
    pub fn new(
        location: Location,
        forecast_days: u8,
        temperature_unit: Option<TemperatureUnit>,
        wind_speed_unit: Option<WindUnit>,
    ) -> Self {
        Self {
            location,
            forecast_days,
            temperature_unit,
            wind_speed_unit,
        }
    }
}

pub type WeatherForecast = Vec<WeatherPoint>;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct WeatherPoint {
    pub timestamp: u64,
    pub temperature: f32,
    pub relative_temperature: f32,
}

impl WeatherPoint {
    pub fn new(timestamp: u64, temperature: f32, relative_temperature: f32) -> Self {
        Self {
            timestamp,
            temperature,
            relative_temperature,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Location {
    pub latitude: f32,
    pub longitude: f32,
}

impl Location {
    pub fn new(latitude: f32, longitude: f32) -> Self {
        Self {
            latitude,
            longitude,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub enum TemperatureUnit {
    Fahrenheit,
    #[default]
    Celsius,
}

impl fmt::Display for TemperatureUnit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let result = match self {
            TemperatureUnit::Fahrenheit => String::from("fahrenheit"),
            TemperatureUnit::Celsius => String::from("celsius"),
        };
        write!(f, "{result}")
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub enum WindUnit {
    #[default]
    Ms,
    Mph,
}

impl fmt::Display for WindUnit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let result = match self {
            WindUnit::Ms => String::from("ms"),
            WindUnit::Mph => String::from("mph"),
        };
        write!(f, "{result}")
    }
}

#[async_trait::async_trait]
pub trait WeatherForecastService {
    async fn forecast(&self, client: &Client, request: ForecastRequest) -> Option<WeatherForecast>;

    fn which_service(&self) -> ForecastService;

    fn change_api_key(&mut self, key: String);
}
