use serde::{Deserialize, Serialize};
use std::error::Error;

// const WEATHER_API:&str = "https://api.data.gov.sg/v1/environment/rainfall";
const WEATHER_API:&str = "https://raw.githubusercontent.com/whkoh/rust-weather/master/src/test_weather.json";
const CONFIG_TOML:&str = "https://raw.githubusercontent.com/whkoh/rust-weather/master/src/weather.toml";

#[derive(Serialize, Deserialize, Debug)]
pub struct Root {
    pub metadata: Metadata,
    pub items: Vec<Item>,
    pub api_info: ApiInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub stations: Vec<Station>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Station {
    pub id: String,
    pub device_id: String,
    pub name: String,
    pub location: Location,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Item {
    pub timestamp: String,
    pub readings: Vec<Reading>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Reading {
    pub station_id: String,
    pub value: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiInfo {
    pub status: String,
}

#[derive(Deserialize, Debug)]
struct Config {
    check: Vec<Check>,
}

#[derive(Deserialize, Debug)]
struct Check {
    stations: Vec<String>,
    // flag: String,
    location: String,
}


fn read_weather() -> Result<Root, Box<dyn Error>> {
    let response = reqwest::blocking::get(WEATHER_API).unwrap();
    let data = serde_json::from_str::<Root>(&*response.text().unwrap())?;
    Ok(data)
}

fn read_config() -> Result<Config, Box<dyn Error>>  {
    let response = reqwest::blocking::get(CONFIG_TOML).unwrap();
    let config = toml::from_str(&*response.text().unwrap())?;
    Ok(config)

}

fn main() {
    let config = match read_config() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to read config: {}", e);
            std::process::exit(1);
        }
    };
    println!("config is: {:?}", config);
    match read_weather() {
        Ok(parsed_data) => {
            for check in config.check {
                let mut rain_stations= Vec::new();
                let enabled_stations = check.stations;
                println!("location is {:?}, \nenabled_stations is\t {:?}", check.location, enabled_stations);
                for item in &parsed_data.items {
                    for reading in &item.readings {
                        if enabled_stations.contains(&reading.station_id) && reading.value > 0.0 {
                            rain_stations.push(&reading.station_id);
                        }
                    }
                }
                println!("rain_stations is:\t {:?}", rain_stations);
            }
        }
        Err(e) => {
            eprintln!("Error parsing JSON: {}", e);
        }
    }

    println!("Hello, world!");
}
