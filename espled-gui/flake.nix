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
            clippy
            rustfmt
            rust-analyzer
          ]);
      in
      {
        devShell = with pkgs; mkShell rec {
          buildInputs = [
            toolchain
            vulkan-loader
            vulkan-validation-layers
            xorg.libX11
            xorg.libxcb
            libxkbcommon
            mesa
            libGL
            libGLU
            pkgs.wayland
            pkgs.wayland-protocols
            pkg-config
            systemd
          ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
          VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d/";
          RUST_LOG = "warn";
        };
      }
    );
}

