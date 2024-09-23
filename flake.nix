{
  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

  # can run on all systems
  inputs.systems.url = "github:nix-systems/default";

  outputs = inputs@{ self, ... }:
  let
    eachSystem = inputs.nixpkgs.lib.genAttrs (import inputs.systems);
  in
  {
    devShells = eachSystem (system:
    let
      pkgs = import inputs.nixpkgs { inherit system; };
    in
    {
      default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          cargo
          pkg-config
        ];
        # fix https://github.com/sfackler/rust-openssl/issues/1663
        PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
      };
    });

    packages = eachSystem (system:
    let
      pkgs = import inputs.nixpkgs { inherit system; };
    in
    {
      lamp-server = pkgs.rustPlatform.buildRustPackage {
        name = "lamp-server";
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;

        nativeBuildInputs = [ pkgs.pkg-config ];
        configurePhase = ''
          runHook preConfigure

          # fix https://github.com/sfackler/rust-openssl/issues/1663
          export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig"

          runHook postConfigure
        '';
      };
    });
  };
}