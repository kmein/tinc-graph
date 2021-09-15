# tinc-graph
`tinc-graph` is a little program to help you display your [Tinc](https://www.tinc-vpn.org/) network graphically.
It was inspired by [makefu/tinc_graphs](https://github.com/makefu/tinc_graphs).

What you need:

1. a Tinc network
2. an IP geolocation database in binary MMDB format (can be downloaded from [MaxMind](https://dev.maxmind.com/geoip/geolite2-free-geolocation-data?lang=en) after creating an account)
3. `tinc-graph`

## How To ...
- ... install: `cargo install`
- ... run the examples:
  1. `sudo tinc-graph --geoip-file PATH_TO_YOUR_GEOIP_DATABASE --network NAME_OF_YOUR_NETWORK > static/network.json`
  2. `python3 -m http.server --directory static` (or similar)
  3. visit <http://0.0.0.0:8000/graph.html> and <http://0.0.0.0:8000/map.html>
