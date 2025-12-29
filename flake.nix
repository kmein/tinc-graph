{
  description = "Generate map data and statistics from a tinc network";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      eachSupportedSystem = lib.genAttrs lib.systems.flakeExposed;
      lib = nixpkgs.lib;
      pkgsFor =
        system:
        import nixpkgs {
          inherit system;
          overlays = [
            rust-overlay.overlays.default
          ];
        };
    in
    {
      devShell = eachSupportedSystem (
        system:
        let
          pkgs = pkgsFor system;
        in
        pkgs.mkShell {
          buildInputs = [
            pkgs.cargo
            pkgs.rustc
            pkgs.rustfmt
            pkgs.jq
            (pkgs.writers.writeDashBin "serve" ''
              ${pkgs.python3}/bin/python3 -m http.server -d static
            '')
          ];
        }
      );
      defaultPackage = eachSupportedSystem (system: self.packages.${system}.tinc-graph);
      packages = eachSupportedSystem (system: {
        tinc-graph = (pkgsFor system).rustPlatform.buildRustPackage {
          name = "tinc-graph";
          src = ./.;
          dontPatchShebangs = 1;
          postInstall = ''
            cp -r $src/static $out
            cp $src/tinc-midpoint $out/bin
          '';
          cargoHash = "sha256-GhDyFhIZDavoAr3182ophbVnqBvpv2cps1k3eCSe0NQ=";
        };
      });
    };
}
