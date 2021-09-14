use tinc_utils::{HostName, Node, TincNetwork};

fn main() {
    let reader = maxminddb::Reader::open_readfile("GeoLite2-City.mmdb").unwrap();
    let retiolum = TincNetwork::new("retiolum");
    let edges = retiolum.get_edges();
    let subnets = retiolum.get_subnets();
    let nodes: std::collections::HashMap<HostName, Node> = retiolum
        .get_nodes()
        .iter()
        .map(|(name, node)| {
            (
                name.to_owned(),
                node.to_owned()
                    .with_edges(&name, &edges)
                    .with_internal_ip(&name, &subnets)
                    .with_location(&reader),
            )
        })
        .collect();

    println!("{}", serde_json::to_string(&nodes).unwrap());
}
