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
    buildInputs = [pkgs.openssl];
    nativeBuildInputs = [pkgs.pkg-config];
  in {
    packages.${system}.default = naerskLib.buildPackage {
      src = ./.;
      buildInputs = buildInputs;
      nativeBuildInputs = nativeBuildInputs;
    };

    devShells.${system}.default = pkgs.mkShell {
      inputsFrom = [self.packages.${system}.default];

      packages = with pkgs; [
        rustc
        cargo
        rustfmt
        rust-analyzer
        zsh
      ];

      shellHook = ''
        export SHELL=${pkgs.zsh}/bin/zsh
        # jump into zsh if we didn't already start in it
        [ -z "$ZSH_VERSION" ] && exec ${pkgs.zsh}/bin/zsh -l
      '';
    };
  };
}
