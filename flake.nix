{
  description = "Generate map data and statistics from a tinc network";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    naersk.url = "github:nix-community/naersk";
    fenix.url = "github:nix-community/fenix";
    naersk.inputs.fenix.follows = "fenix";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      fenix,
      naersk,
    }:
    let
      eachSupportedSystem = lib.genAttrs lib.systems.flakeExposed;
      lib = nixpkgs.lib;
      pkgsFor =
        system:
        import nixpkgs {
          inherit system;
          overlays = [ ];
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
      packages = eachSupportedSystem (
        system:
        let
          pkgs = pkgsFor system;
          naersk' = pkgs.callPackage naersk { };
        in
        {
          tinc-graph = naersk'.buildPackage {
            name = "tinc-graph";
            src = ./.;
            dontPatchShebangs = 1;
            postInstall = ''
              cp -r $src/static $out
              cp $src/tinc-midpoint $out/bin
            '';
            cargoHash = "sha256-GhDyFhIZDavoAr3182ophbVnqBvpv2cps1k3eCSe0NQ=";
          };
        }
      );
    };
}
