{
  description = "lead";

  inputs = {
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    naersk,
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
        };
        naerskLib = pkgs.callPackage naersk {};
        buildInputs = [pkgs.openssl];
        nativeBuildInputs = [pkgs.pkg-config];
        lead = naerskLib.buildPackage {
          src = ./.;
          buildInputs = buildInputs;
          nativeBuildInputs = nativeBuildInputs;
        };
      in {
        packages.default = lead;

        devShells.default = pkgs.mkShell {
          inputsFrom = [lead];

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
      }
    );
}
