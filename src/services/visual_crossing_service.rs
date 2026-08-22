use super::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const BASE_URL: &str =
    "https://weather.visualcrossing.com/VisualCrossingWebServices/rest/services/timeline";

pub struct visual_crossing_service<'a> {
    pub client: &'a Client,
    pub api_key: String,
}

impl<'a> visual_crossing_service<'a> {
    pub fn new(client: &'a Client, api_key: String) -> Self {
        Self { client, api_key }
    }

    pub fn build_url(location: Location) -> String {
        format!("{}/{},{}", BASE_URL, location.latitude, location.longitude)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_base_url_builder() {
        let location = Location::new(55.7558, 37.6173);
        let final_url = visual_crossing_service::build_url(location);
        let test = "https://weather.visualcrossing.com/VisualCrossingWebServices/rest/services/timeline/55.7558,37.6173";
        assert_eq!(final_url, test);
    }

    #[test]
    fn check_final_url() {}
}
