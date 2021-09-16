{ lib, rustPlatform }:
rustPlatform.buildRustPackage rec {
  name = "tinc-graph";
  src = ./.;
  cargoSha256 = "1v8fydpqgcv52fcvkpw6j9cx2xhdryn98b91f3d53ph2qqxj25qn";
}
