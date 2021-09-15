use structopt::StructOpt;
use tinc_utils::{HostName, Node, TincNetwork};

#[derive(StructOpt, Debug)]
#[structopt(name = "tinc_graph")]
struct Opt {
    #[structopt(short, long)]
    network: String,
    #[structopt(long)]
    geoip_file: String,
}

fn main() {
    let opt = Opt::from_args();

    let reader =
        maxminddb::Reader::open_readfile(opt.geoip_file).expect("Failed to open GeoIP database");
    let retiolum = TincNetwork::new(&opt.network);
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
