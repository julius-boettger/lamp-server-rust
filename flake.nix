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
      lamp-server-pkg =
        {
          # govee secrets that should be overridden
          govee_api_key ? "",
          govee_device  ? "",
          govee_model   ? ""
        }:
        pkgs.rustPlatform.buildRustPackage {
          name = "lamp-server";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = [ pkgs.pkg-config ];
          configurePhase = ''
            runHook preConfigure

            # fix https://github.com/sfackler/rust-openssl/issues/1663
            export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig"

            cat <<EOF > src/constants/govee_secrets.rs
              pub const API_KEY: &str = "${govee_api_key}";
              pub const DEVICE:  &str = "${govee_device}";
              pub const MODEL:   &str = "${govee_model}";
            EOF

            runHook postConfigure
          '';
        };
    in
    {
      lamp-server = pkgs.callPackage lamp-server-pkg {};
    });
  };
}