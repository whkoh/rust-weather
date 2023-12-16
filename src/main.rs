use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::error::Error;

// const WEATHER_API:&str = "https://api.data.gov.sg/v1/environment/rainfall";
const WEATHER_API:&str = "https://raw.githubusercontent.com/whkoh/rust-weather/master/src/test_weather.json";
const CONFIG_TOML:&str = "https://raw.githubusercontent.com/whkoh/rust-weather/master/src/weather.toml";
const FLAGS_API:&str = "https://cdn.growthbook.io/api/features/prod_cbAOvmDygOHhAGOsaKdMhs7lI3wfI9bNAIqIeyNhos";

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
    flag: String,
    location: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Flags {
    status: u32,
    features: HashMap<String, Feature>,
    dateUpdated: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum Feature {
    Bool { defaultValue: bool },
    String { defaultValue: String },
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

fn read_flags() -> Result<Vec<String>, Box<dyn Error>> {
    let response = reqwest::blocking::get(FLAGS_API).unwrap();
    let flags = serde_json::from_str::<Flags>(&*response.text().unwrap())?;
    let features = flags.features;
    let mut enabled_flags = Vec::new();
    for feature in features {
        // println!("feature is: {:?}",feature.0);
        enabled_flags.push(feature.0);
    }
    Ok(enabled_flags)
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
    let flags = match read_flags() {
        Ok(flags) => flags,
        Err(e) => {
            eprintln!("Failed to read flags: {}", e);
            std::process::exit(1);
        }
    };
    println!("flags is: {:?}", flags);
    let parsed_data = match read_weather() {
        Ok(parsed_data) => parsed_data,
        Err(e) => {
            eprintln!("Error parsing JSON: {}", e);
            std::process::exit(1);
        }
    };
    for check in config.check {
        if !&flags.contains(&check.flag) {
            continue
        }
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
        // println!("rain_stations is:\t {:?}", rain_stations);
        if !rain_stations.is_empty() {
            println!("It's raining in {}", check.location);
            continue;
        }
        println!("It's NOT raining in {}", check.location);
    }
    println!("Hello, world!");
}
