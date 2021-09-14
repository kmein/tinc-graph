use maxminddb::{geoip2::City, Reader};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::net::IpAddr;

pub type Port = u16;
pub type HostName = String;

pub struct TincNetwork {
    name: String,
}

impl TincNetwork {
    pub fn new(name: &str) -> Self {
        TincNetwork {
            name: name.to_string(),
        }
    }

    fn run_command(&self, command: &str) -> String {
        let mut tinc_args = vec!["-n", &self.name];
        tinc_args.extend(command.split_whitespace());
        let output = std::process::Command::new("tinc")
            .args(tinc_args)
            .output()
            .expect("failed to execute tinc command");
        String::from_utf8(output.stdout).expect("found invalid utf8")
    }

    pub fn get_nodes(&self) -> HashMap<HostName, Node> {
        let mut result = HashMap::new();
        for line in self.run_command("dump reachable nodes").lines() {
            let words: Vec<_> = line.split_whitespace().collect();
            result.insert(
                words[0].to_string(),
                Node::new(
                    words[4].parse().ok(),
                    words[6].parse().expect("Invalid port number"),
                ),
            );
        }
        result
    }

    pub fn get_subnets(&self) -> HashMap<IpAddr, HostName> {
        let mut result = HashMap::new();
        for line in self.run_command("dump subnets").lines() {
            let words: Vec<_> = line.split_whitespace().collect();
            match words.as_slice() {
                [address_str, "owner", name] => {
                    let subnet: Vec<_> = address_str.split("/").collect();
                    if name == &"(broadcast)" {
                        continue;
                    }
                    result.insert(
                        subnet[0].parse().expect("Invalid IP address"),
                        name.to_string(),
                    );
                }
                _ => panic!("Erroneous output from tinc: {}", line),
            }
        }
        result
    }

    pub fn get_edges(&self) -> HashMap<HostName, Vec<Edge>> {
        let mut result: HashMap<String, Vec<Edge>> = HashMap::new();

        for line in self.run_command("dump edges").lines() {
            let words: Vec<_> = line.split_whitespace().collect();
            match words.as_slice() {
                [from_name, "to", to_name, "at", address_str, "port", port_str, "local", _local_address_str, "port", _local_port_str, "options", _options, "weight", weight_str] => {
                    result.entry(from_name.to_string()).or_default().push(Edge {
                        name: to_name.to_string(),
                        address: address_str.parse().expect("Invalid IP address"),
                        port: port_str.parse().expect("Invalid port"),
                        weight: weight_str.parse().expect("Invalid weight"),
                    })
                }
                _ => panic!("Erroneous output from tinc: {}", line),
            }
        }

        result
    }
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all(serialize = "kebab-case"))]
pub struct Node {
    external_ip: Option<IpAddr>,
    external_port: Port,
    internal_ip: Vec<IpAddr>,
    to: Vec<Edge>,
    country: Option<String>,
    city: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

impl Node {
    pub fn new(external_ip: Option<IpAddr>, external_port: Port) -> Self {
        Node {
            external_ip: external_ip,
            external_port: external_port,
            to: Vec::new(),
            internal_ip: Vec::new(),
            city: None,
            country: None,
            latitude: None,
            longitude: None,
        }
    }

    pub fn with_internal_ip(self, name: &HostName, subnets: &HashMap<IpAddr, HostName>) -> Self {
        Node {
            internal_ip: subnets
                .iter()
                .filter_map(|(key, val)| {
                    if val == name {
                        Some(key.to_owned())
                    } else {
                        None
                    }
                })
                .collect(),
            ..self
        }
    }

    pub fn with_edges(self, name: &HostName, edges: &HashMap<HostName, Vec<Edge>>) -> Self {
        fn resolve(name: &HostName, edges: &HashMap<HostName, Vec<Edge>>) -> Option<IpAddr> {
            let mut result = None;
            for (_, node_edges) in edges {
                for edge in node_edges {
                    if &edge.name == name {
                        result = Some(edge.address);
                        break;
                    }
                }
            }
            result
        }
        Node {
            external_ip: self.external_ip.or_else(|| resolve(&name, edges)),
            to: edges
                .get(name)
                .map(|node_edges| node_edges.iter().map(|edge| edge.to_owned()).collect())
                .unwrap_or_else(Vec::new),
            ..self
        }
    }

    pub fn with_location(self, reader: &Reader<Vec<u8>>) -> Self {
        fn english_name(names: BTreeMap<&str, &str>) -> Option<String> {
            names.get("en").map(|name| name.to_string())
        }

        let maybe_city: Option<City> = self
            .external_ip
            .and_then(|external_ip| reader.lookup(external_ip).ok());
        let maybe_location = maybe_city.clone().and_then(|city| city.location);
        Node {
            latitude: maybe_location
                .clone()
                .and_then(|location| location.latitude),
            longitude: maybe_location.and_then(|location| location.longitude),
            city: maybe_city
                .clone()
                .and_then(|city| city.city)
                .and_then(|city| city.names)
                .and_then(english_name),
            country: maybe_city
                .and_then(|city| city.country)
                .and_then(|country| country.names)
                .and_then(english_name),
            ..self
        }
    }
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all(serialize = "kebab-case"))]
pub struct Edge {
    name: String,
    address: IpAddr,
    port: Port,
    weight: i16,
}
