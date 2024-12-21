{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    systems.url = "github:nix-systems/default"; # can run on all systems
  };

  outputs = { self, nixpkgs, systems, ... }:
  let
    eachSystem = fn: nixpkgs.lib.genAttrs (import systems) (system: fn system (import nixpkgs {
      inherit system;
    }));
  in
  {
    packages = eachSystem (system: pkgs: rec {
      default = lamp-server;
      lamp-server = pkgs.rustPlatform.buildRustPackage {
        name = "lamp-server";
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;

        nativeBuildInputs = [ pkgs.pkg-config ];
              buildInputs = [ pkgs.openssl ];
      };
    });

    devShells = eachSystem (system: pkgs: {
      default = let
        inherit (self.packages.${system}.default) nativeBuildInputs buildInputs;
      in pkgs.mkShell {
        buildInputs = buildInputs;
        nativeBuildInputs = nativeBuildInputs ++ [ pkgs.cargo ];
        RUSTFLAGS = "--cfg govee_debug";
      };
    });
  };
}