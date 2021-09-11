use std::collections::hash_map::HashMap;
use std::net::IpAddr;
use tinc_utils::{Edge, HostName, Node};

fn run_tinc(network: &str, args: Vec<&str>) -> String {
    let mut tinc_args = vec!["-n", &network];
    tinc_args.extend(args);
    let output = std::process::Command::new("tinc")
        .args(tinc_args)
        .output()
        .expect("failed to execute tinc command");
    String::from_utf8(output.stdout).expect("found invalid utf8")
}

fn get_nodes(network: &str) -> HashMap<HostName, Node> {
    let mut result = HashMap::new();

    for line in run_tinc(network, vec!["dump", "reachable", "nodes"]).lines() {
        let words: Vec<_> = line.split_whitespace().collect();
        result.insert(
            words[0].to_string(),
            Node {
                external_ip: words[4].parse().ok(),
                external_port: words[6].parse().expect("Invalid port number"),
                to: Vec::new(),
                internal_ip: Vec::new(),
            },
        );
    }

    result
}

fn get_subnets(network: &str) -> HashMap<IpAddr, HostName> {
    let mut result = HashMap::new();

    for line in run_tinc(network, vec!["dump", "subnets"]).lines() {
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

fn get_edges(network: &str) -> HashMap<HostName, Vec<Edge>> {
    let mut result: HashMap<String, Vec<Edge>> = HashMap::new();

    for line in run_tinc(network, vec!["dump", "edges"]).lines() {
        let words: Vec<_> = line.split_whitespace().collect();
        match words.as_slice() {
            [from_name, "to", to_name, "at", address_str, "port", port_str, "local", _local_address_str, "port", _local_port_str, "options", _options, "weight", weight_str] => {
                result.entry(from_name.to_string()).or_default().push(Edge {
                    name: to_name.to_string(),
                    addr: address_str.parse().expect("Invalid IP address"),
                    port: port_str.parse().expect("Invalid port"),
                    weight: weight_str.parse().expect("Invalid weight"),
                })
            }
            _ => panic!("Erroneous output from tinc: {}", line),
        }
    }

    result
}

fn main() {
    let network = "retiolum";
    let edges = get_edges(network);
    let subnets = get_subnets(network);

    let mut result = HashMap::new();

    for (name, node) in get_nodes(network) {
        result.insert(
            name.to_string(),
            Node {
                internal_ip: subnets
                    .iter()
                    .filter_map(|(key, val)| {
                        if val == &name {
                            Some(key.clone())
                        } else {
                            None
                        }
                    })
                    .collect(),
                to: if let Some(value) = edges.get(&name) {
                    value.iter().map(|x| x.clone()).collect()
                } else {
                    Vec::new()
                },
                ..node
            },
        );
    }

    println!("{}", serde_json::to_string(&result).unwrap());
}
