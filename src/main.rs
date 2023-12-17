use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::error::Error;

use chrono::{DateTime, Local};
use reqwest::blocking::Client;
use simplelog::*;

use std::fs::OpenOptions;
use tokio;

const WEATHER_API:&str = "https://api.data.gov.sg/v1/environment/rainfall";
// const WEATHER_API: &str = "https://raw.githubusercontent.com/whkoh/rust-weather/master/src/test_weather.json";
const CONFIG_TOML: &str = "https://raw.githubusercontent.com/whkoh/rust-weather/master/src/weather.toml";
const FLAGS_API: &str = "https://cdn.growthbook.io/api/features/prod_cbAOvmDygOHhAGOsaKdMhs7lI3wfI9bNAIqIeyNhos";
const NOTIFY_API: &str = "https://ntfy.sh/aek6Igha6ia";

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
#[allow(non_snake_case)]
struct Flags {
    status: u32,
    features: HashMap<String, Feature>,
    dateUpdated: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
#[allow(non_snake_case)]
enum Feature {
    Bool { defaultValue: bool },
    String { defaultValue: String },
}

async fn read_weather() -> Result<Root, Box<dyn Error + Send + Sync>> {
    let response = reqwest::get(WEATHER_API).await?;
    let text = response.text().await?;
    let data = serde_json::from_str::<Root>(&text)?;
    Ok(data)
}

async fn read_config() -> Result<Config, Box<dyn Error + Send + Sync>> {
    let response = reqwest::get(CONFIG_TOML).await?;
    let text = response.text().await?;
    let config = toml::from_str(&text)?;
    Ok(config)
}

async fn read_flags() -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
    let response = reqwest::get(FLAGS_API).await?;
    let text = response.text().await?;
    let flags = serde_json::from_str::<Flags>(&text)?;
    let features = flags.features;
    let mut enabled_flags = Vec::new();
    for feature in features {
        enabled_flags.push(feature.0);
    }
    Ok(enabled_flags)
}

fn notify(message: String) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let res = client.post(NOTIFY_API)
        .body(message.to_string())
        .send()?;
    if !res.status().is_success() {
        log::error!("Request failed with status: {}", res.status());
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let log_file = OpenOptions::new()
        .write(true)
        .create(true)  // Create the file if it does not exist
        .append(true)  // Append to the file if it exists
        .open("my_log.log")
        .unwrap();
    let config = ConfigBuilder::new()
        .set_time_format_custom(format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]"))
        .build();
    let _ = WriteLogger::init(
        LevelFilter::Debug,
        config,
        log_file
    );
    let (config_result, flags_result, weather_result) = tokio::join!(
        tokio::task::spawn(read_config()),
        tokio::task::spawn(read_flags()),
        tokio::task::spawn(read_weather())
    );
    let config = match config_result {
        Ok(Ok(config)) => config,
        Ok(Err(e)) => {
            log::error!("Failed to read config: {}", e);
            std::process::exit(1);
        },
        Err(e) => {
            log::error!("Task failed: {}", e);
            std::process::exit(1);
        }
    };
    log::debug!("config is: {:?}", config);
    let flags = match flags_result {
        Ok(Ok(flags)) => flags,
        Ok(Err(e)) => {
            log::error!("Failed to read flags: {}", e);
            std::process::exit(1);
        },
        Err(e) => {
            log::error!("Task failed: {}", e);
            std::process::exit(1);
        }
    };
    log::debug!("flags is: {:?}", flags);
    let parsed_data = match weather_result {
        Ok(Ok(parsed_data)) => parsed_data,
        Ok(Err(e)) => {
            log::error!("Error parsing JSON: {}", e);
            std::process::exit(1);
        },
        Err(e) => {
            log::error!("Task failed: {}", e);
            std::process::exit(1);
        }
    };
    let now: DateTime<Local> = Local::now();
    for check in config.check {
        let formatted_time = now.format("%a %Y-%m-%d @ %H:%M").to_string();
        if !&flags.contains(&check.flag) {
            continue;
        }
        let mut rain_amount = Vec::new();
        let enabled_stations = check.stations;
        log::debug!("location is {:?}, \nenabled_stations is\t {:?}", check.location, enabled_stations);
        for item in &parsed_data.items {
            for reading in &item.readings {
                if !(enabled_stations.contains(&reading.station_id) && reading.value > 0.0) {
                    continue;
                }
                rain_amount.push(&reading.value);
            }
        }
        if !rain_amount.is_empty() {
            let smallest = rain_amount.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
            let sum: f32 = rain_amount.iter().map(|&num| num).sum();
            let count: f32 = rain_amount.len() as f32;
            let average: f32 = sum / count;
            notify(format!("💧{} ({} stn) at {}. Min: {:?}mm, Avg: {:?}mm",
                           check.location, count, formatted_time, smallest, average
            )).expect("Notification failed");
            continue;
        }
        log::info!("It's NOT raining in {}", check.location);
    }
}
