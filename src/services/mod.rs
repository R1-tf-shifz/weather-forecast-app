use serde::{Deserialize, Serialize};
use std::fmt;

use crate::services::{
    open_meteo_service::OpenMeteoService, visual_crossing_service::VisualCrossingService,
};

pub mod open_meteo_service;
pub mod visual_crossing_service;

pub enum ForecastService<'a> {
    VisualCrossing(VisualCrossingService<'a>),
    OpenMeteo(OpenMeteoService<'a>),
}

impl<'a> ForecastService<'a> {
    pub fn change_api_key(&mut self, key: String) {
        match self {
            Self::VisualCrossing(service) => service.api_key = key,
            Self::OpenMeteo(_) => (),
        }
    }
}

impl<'a> fmt::Display for ForecastService<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result = match self {
            Self::VisualCrossing(_) => "VisualCrossing".to_string(),
            Self::OpenMeteo(_) => "OpenMeteo".to_string(),
        };
        write!(f, "{result}")
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ForecastRequest {
    pub location: Location,
    pub forecast_days: u8,
    pub api_key: Option<String>,
    pub temperature_unit: Option<TemperatureUnit>,
    pub wind_speed_unit: Option<WindUnit>,
}

impl ForecastRequest {
    pub fn new(
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

pub type WeatherForecast = Vec<WeatherPoint>;

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug, Default)]
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

#[derive(Serialize, Deserialize, Debug, Default)]
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
    async fn forecast(&self, request: ForecastRequest) -> Option<WeatherForecast>;
}
