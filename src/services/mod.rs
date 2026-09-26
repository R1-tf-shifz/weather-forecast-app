use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fmt;

const IP_API_URL: &str = "http://ip-api.com/json/?fields=status,lat,lon";

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

    pub async fn with_ip_api_location(
        forecast_days: u8,
        temperature_unit: Option<TemperatureUnit>,
        wind_speed_unit: Option<WindUnit>,
        client: &Client,
    ) -> Option<Self> {
        let location = Location::ip_api_location(client).await?;
        Some(Self::new(
            location,
            forecast_days,
            temperature_unit,
            wind_speed_unit,
        ))
    }
}

pub type WeatherForecast = Vec<WeatherPoint>;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Wmo {
    //wmo codes
    NoPrecipitation,
    Drizzle,
    Rain,
    Snow,
    RainShower,
    SnowShower,
    ThunderStorm,
}

impl Wmo {
    fn from_i32(code: i32) -> Self {
        match code {
            0..10 => Self::NoPrecipitation,
            50..=59 => Self::Drizzle,
            60..=69 => Self::Rain,
            70..=79 => Self::Snow,
            80..=82 => Self::RainShower,
            85..86 => Self::SnowShower,
            95..100 => Self::ThunderStorm,
            _ => Self::default(),
        }
    }
}

impl Default for Wmo {
    fn default() -> Self {
        Self::NoPrecipitation
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct WeatherPoint {
    pub timestamp: u64,
    pub temperature: f32,
    pub relative_temperature: f32,
    pub weather_code: Option<Wmo>,
}

impl WeatherPoint {
    pub fn new(
        timestamp: u64,
        temperature: f32,
        relative_temperature: f32,
        weather_code: Option<Wmo>,
    ) -> Self {
        Self {
            timestamp,
            temperature,
            relative_temperature,
            weather_code,
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

    pub async fn ip_api_location(client: &Client) -> Option<Self> {
        #[derive(Serialize, Deserialize)]
        struct ApiResponse {
            status: String,
            lat: f32,
            lon: f32,
        }

        let response = client.get(IP_API_URL).send().await.ok()?;
        let json = response.json::<ApiResponse>().await.ok()?;
        if json.status == "success" {
            Some(Location::new(json.lat, json.lon))
        } else {
            None
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_ip_api_location() {
    let client = Client::new();
    let result = Location::ip_api_location(&client).await;
    assert!(result.is_some())
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
