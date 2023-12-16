use serde::{Deserialize, Serialize};

const WEATHER_API:&str = "https://api.data.gov.sg/v1/environment/rainfall";

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
    pub value: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiInfo {
    pub status: String,
}


fn read_weather() {
    let response = reqwest::blocking::get(WEATHER_API).unwrap();
    match serde_json::from_str::<Root>(&*response.text().unwrap()) {
        Ok(parsed_data) => {
            println!("{:#?}", parsed_data);
        }
        Err(e) => {
            eprintln!("Error parsing JSON: {}", e);
        }
    }


}

fn main() {
    read_weather();
    println!("Hello, world!");
}
