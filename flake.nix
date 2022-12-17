{
  inputs = {
    naersk = {
      url = "github:nix-community/naersk/master";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:pupbrained/fenix/patch-1";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    naersk,
    fenix,
  }:
    utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
      naersk-lib = pkgs.callPackage naersk {};
    in {
      defaultPackage = let
        pkgs = nixpkgs.legacyPackages.${system};
        target = "x86_64-unknown-linux-gnu";
        toolchain = with fenix.packages.${system};
          combine [
            minimal.cargo
            minimal.rustc
            targets.${target}.latest.rust-std
          ];
      in
        (naersk.lib.${system}.override {
          cargo = toolchain;
          rustc = toolchain;
        })
        .buildPackage {
          src = ./.;
          buildInputs = with pkgs; [dbus];
          nativeBuildInputs = with pkgs; [pkg-config];
          CARGO_BUILD_TARGET = target;
          CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER = "${pkgs.stdenv.cc}/bin/${target}-gcc";
        };

      defaultApp = utils.lib.mkApp {
        drv = self.defaultPackage."${system}";
      };

      devShell = with pkgs;
        mkShell {
          buildInputs = [dbus];
          nativeBuildInputs = [pkg-config];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
    });
}
