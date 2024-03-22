# run with `nix develop [-c cargo run]`
# if on WSL: spaces in absolute path to this directory are not allowed :)
{
  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  outputs = inputs@{ self, ... }:
  let
    system = "x86_64-linux";
    pkgs-config = { inherit system; };
    pkgs = import inputs.nixpkgs pkgs-config;
  in
  {
    devShells.${system}.default = pkgs.mkShell {
      nativeBuildInputs = with pkgs; [
        cargo
        pkg-config
        openssl.dev
      ];
      # fix https://github.com/sfackler/rust-openssl/issues/1663
      PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
    };
  };
}