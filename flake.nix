{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix";
  };

  outputs = { self, nixpkgs, utils, fenix }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

        toolchain = with fenix.packages.${system};
          combine (with complete; [
            rustc
            cargo
            rust-src
            clippy
            rustfmt
            rust-analyzer
          ]);
        riscvToolchain = fenix.packages.${system}.targets.riscv32imc-unknown-none-elf.latest.rust-std;
      in
      {
        devShell = with pkgs; mkShell rec {
          buildInputs = [
            toolchain
            riscvToolchain
            espflash
            python3
            llvmPackages_19.libclang
            ldproxy
          ];

          #RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
          LIBCLANG_PATH = "${pkgs.llvmPackages_19.libclang.lib}/lib";
          RUST_BACKTRACE = 1;
        };
      }
    );
}

