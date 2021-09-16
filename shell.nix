{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  buildInputs = with pkgs; [
    cargo
    rustc
    rustfmt
    jq
    (pkgs.writers.writeDashBin "serve" ''
      ${pkgs.python3}/bin/python3 -m http.server -d static
    '')
  ];
}
