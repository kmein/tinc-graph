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

    fn run_command(&self, args: Vec<&str>) -> String {
        let mut tinc_args = vec!["-n", &self.name];
        tinc_args.extend(args);
        let output = std::process::Command::new("tinc")
            .args(tinc_args)
            .output()
            .expect("failed to execute tinc command");
        String::from_utf8(output.stdout).expect("found invalid utf8")
    }

    pub fn get_nodes(&self) -> HashMap<HostName, Node> {
        let mut result = HashMap::new();
        for line in self.run_command(vec!["dump", "reachable", "nodes"]).lines() {
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
        for line in self.run_command(vec!["dump", "subnets"]).lines() {
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

        for line in self.run_command(vec!["dump", "edges"]).lines() {
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
                .filter_map(
                    |(key, val)| {
                        if val == name {
                            Some(key.clone())
                        } else {
                            None
                        }
                    },
                )
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
            to: if let Some(value) = edges.get(name) {
                value.iter().map(|x| x.clone()).collect()
            } else {
                Vec::new()
            },
            ..self
        }
    }

    pub fn with_location(self, reader: &Reader<Vec<u8>>) -> Self {
        fn get_english_name(maybe_names: Option<BTreeMap<&str, &str>>) -> Option<String> {
            maybe_names.and_then(|names| {
                if let Some(english_name) = names.get("en") {
                    Some(english_name.to_string())
                } else {
                    None
                }
            })
        }

        if let Some(external_ip) = self.external_ip {
            let maybe_city: Option<City> = reader.lookup(external_ip).ok();
            let maybe_location = maybe_city.clone().and_then(|city| city.location);
            Node {
                latitude: maybe_location
                    .clone()
                    .and_then(|location| location.latitude),
                longitude: maybe_location.and_then(|location| location.longitude),
                city: get_english_name(
                    maybe_city
                        .clone()
                        .and_then(|city| city.city)
                        .and_then(|city| city.names),
                ),
                country: get_english_name(
                    maybe_city
                        .and_then(|city| city.country)
                        .and_then(|country| country.names),
                ),
                ..self
            }
        } else {
            self
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
