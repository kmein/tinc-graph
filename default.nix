{ lib, rustPlatform, jq }:
rustPlatform.buildRustPackage rec {
  name = "tinc-graph";
  src = ./.;
  dontPatchShebangs = 1;
  postInstall = ''
    cp -r $src/static $out
    cp $src/tinc-statistics $out/bin
  '';
  cargoSha256 = "1v8fydpqgcv52fcvkpw6j9cx2xhdryn98b91f3d53ph2qqxj25qn";
}
