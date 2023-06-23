{
  description = "Generate map data and statistics from a tinc network";

  inputs = {
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.follows = "rust-overlay/flake-utils";
    nixpkgs.follows = "rust-overlay/nixpkgs";
  };

  outputs = inputs: with inputs;
    flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          rust-overlay.overlays.default
        ];
      };
    in
    {
      devShell = pkgs.mkShell {
        buildInputs = with pkgs; [
          cargo
          rustc
          rustfmt
          jq
          (pkgs.writers.writeDashBin "serve" ''
            ${pkgs.python3}/bin/python3 -m http.server -d static
          '')
        ];
      };
      defaultPackage = self.packages.${system}.tinc-graph;
      packages.tinc-graph = pkgs.rustPlatform.buildRustPackage rec {
        name = "tinc-graph";
        src = ./.;
        dontPatchShebangs = 1;
        postInstall = ''
          cp -r $src/static $out
          cp $src/tinc-midpoint $out/bin
        '';
        cargoSha256 = "1v8fydpqgcv52fcvkpw6j9cx2xhdryn98b91f3d53ph2qqxj25qn";
      };
    });
}
