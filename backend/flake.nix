{
  description = "lead-rust";

  inputs = {
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = {
    self,
    naersk,
    nixpkgs,
  }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {inherit system;};
    naerskLib = pkgs.callPackage naersk {};
  in {
    packages.${system}.default = naerskLib.buildPackage {
      src = ./.;
      buildInputs = [pkgs.openssl];
      nativeBuildInputs = [pkgs.pkg-config];
    };
  };
}
