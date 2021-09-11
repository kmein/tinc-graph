use serde::Serialize;
use std::net::IpAddr;

pub type Port = u16;
pub type HostName = String;

#[derive(Serialize, Clone)]
#[serde(rename_all(serialize = "kebab-case"))]
pub struct Node {
    pub external_ip: Option<IpAddr>,
    pub external_port: Port,
    pub internal_ip: Vec<IpAddr>,
    pub to: Vec<Edge>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all(serialize = "kebab-case"))]
pub struct Edge {
    pub name: String,
    pub addr: IpAddr,
    pub port: Port,
    pub weight: i16,
}

struct Location {
    dma_code: u8,
    area_code: u8,
    metro_code: u32,
    postal_code: u32,
    country_code: String,
    country_code3: String,
    country_name: String,
    continent: String,
    region_code: String,
    city: String,
    time_zone: String,
    position: Position,
}

struct Position {
    latitude: f64,
    longitude: f64,
}
