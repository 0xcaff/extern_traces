{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
    crate2nix.url = "github:nix-community/crate2nix";
    fenix.url = "github:nix-community/fenix";
    treefmt-nix.url = "github:numtide/treefmt-nix";

    pyproject-nix = {
      url = "github:pyproject-nix/pyproject.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    uv2nix = {
      url = "github:pyproject-nix/uv2nix";
      inputs.pyproject-nix.follows = "pyproject-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    pyproject-build-systems = {
      url = "github:pyproject-nix/build-system-pkgs";
      inputs.pyproject-nix.follows = "pyproject-nix";
      inputs.uv2nix.follows = "uv2nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    ps-nix.url = "github:0xcaff/ps-nix";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      crate2nix,
      fenix,
      treefmt-nix,
      pyproject-nix,
      uv2nix,
      pyproject-build-systems,
      ps-nix,
      ...
    }:
    let
      supported-systems = with flake-utils.lib.system; [
        x86_64-linux
        aarch64-darwin
      ];
    in
    flake-utils.lib.eachSystem supported-systems (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            fenix.overlays.default
            crate2nix.overlays.default
          ];
        };

        pythonCodegenEnv =
          let
            uvWorkspace = uv2nix.lib.workspace.loadWorkspace {
              workspaceRoot = ./.;
            };

            overlay = uvWorkspace.mkPyprojectOverlay {
              sourcePreference = "wheel";
            };

            pyprojectOverrides = _final: prev: {
              # mako should work out of the box, but add overrides here if needed
            };

            pythonSet =
              (pkgs.callPackage pyproject-nix.build.packages {
                python = pkgs.python312;
              }).overrideScope
                (
                  pkgs.lib.composeManyExtensions [
                    pyproject-build-systems.overlays.default
                    overlay
                    pyprojectOverrides
                  ]
                );

          in
          pythonSet.mkVirtualEnv "extern-traces-codegen" uvWorkspace.deps.all;

        rustToolchainForPkgs = (
          pkgs:
          fenix.packages.${system}.fromToolchainFile {
            file = ./rust-toolchain.toml;
            sha256 = "sha256-SdELfyScVKtHr4qEIxY59QFcFR8tolVWN8rkc8YLyOw=";
          }
        );

        sharedCrateOverrides = pkgs.defaultCrateOverrides // {
          pm4 = _: {
            prePatch = ''
              pushd src/registers/generated
              ${pythonCodegenEnv}/bin/python regs_rs.py
              ${pythonCodegenEnv}/bin/python pkt3_rs.py
              popd
            '';
          };

          gcn = _: {
            prePatch = ''
              pushd src/instructions/generated
              ${pythonCodegenEnv}/bin/python ops_rs.py
              popd
            '';
          };

          ps4libdoc = _: {
            prePatch = ''
              substituteInPlace src/lib.rs \
                --replace-fail "defs" "${
                  pkgs.fetchFromGitHub {
                    owner = "idc";
                    repo = "ps4libdoc";
                    rev = "a71315e7f36e312ae71e9e3a92982e9ffbfc725f";
                    sha256 = "sha256-33wVp2eBsPf42k25dGKMHGMFqnSwXthoF5Bg/o30e/M=";
                  }
                }"
            '';
          };

          shaderc-sys = _: {
            buildInputs = [
              pkgs.cmake
              pkgs.git
              pkgs.python312
            ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.darwin.cctools
            ];
          };

          extern_traces_viewer = attrs: {
            GIT_SHA_SHORT = self.rev or "unknown";
            nativeBuildInputs = (attrs.nativeBuildInputs or [ ]) ++ [
              pkgs.pkg-config
              pkgs.cmake
              pkgs.ninja
            ];
            buildInputs =
              (attrs.buildInputs or [ ])
              ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
                pkgs.vulkan-loader
                pkgs.vulkan-headers
                pkgs.vulkan-validation-layers
                pkgs.xorg.libX11
                pkgs.xorg.libXcursor
                pkgs.xorg.libXi
                pkgs.xorg.libXrandr
                pkgs.libGL
                pkgs.fontconfig
                pkgs.freetype
                pkgs.libxkbcommon
                pkgs.wayland
              ]
              ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
                pkgs.darwin.apple_sdk.frameworks.Metal
                pkgs.darwin.apple_sdk.frameworks.QuartzCore
                pkgs.darwin.apple_sdk.frameworks.Cocoa
                pkgs.darwin.apple_sdk.frameworks.AppKit
                pkgs.darwin.apple_sdk.frameworks.CoreGraphics
              ];
          };

          vulkano = attrs: {
            nativeBuildInputs = (attrs.nativeBuildInputs or [ ]) ++ [
              pkgs.vulkan-headers
            ];
            buildInputs =
              (attrs.buildInputs or [ ])
              ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
                pkgs.vulkan-loader
              ];
          };

          vulkano-shaders = attrs: {
            nativeBuildInputs = (attrs.nativeBuildInputs or [ ]) ++ [
              pkgs.vulkan-headers
            ];
          };
        };

        cargoProject = pkgs.callPackage ./Cargo.nix {
          defaultCrateOverrides = sharedCrateOverrides;
        };

        pkgsFbsd = import nixpkgs {
          inherit system;
          crossSystem = {
            config = "x86_64-unknown-freebsd";
          };
          overlays = [
            fenix.overlays.default
            crate2nix.overlays.default
            (final: prev: {
              bmake = prev.bmake.overrideAttrs (old: {
                preConfigure =
                  (old.preConfigure or "")
                  + ''
                    # expose wchar_t and use a modern C dialect
                    export NIX_CFLAGS_COMPILE="$NIX_CFLAGS_COMPILE -std=gnu99 -D_DARWIN_C_SOURCE"

                    # the autodetection on Apple silicon picks MACHINE_ARCH=arm (wrong)
                    export MACHINE_ARCH=arm64
                    export MACHINE=arm64        # keeps bmake’s paths consistent
                  '';
              });
            })

          ];
        };

        pluginSupportProject =
          pkgsFbsd.callPackage ./packages/extern_traces_plugin/plugin_support/Cargo.nix
            {
              defaultCrateOverrides = sharedCrateOverrides;
              buildRustCrateForPkgs = (
                pkgs:
                pkgs.buildRustCrate.override {
                  rustc = (rustToolchainForPkgs pkgs);
                  cargo = (rustToolchainForPkgs pkgs);
                }
              );
            };

        extern_traces_plugin = pkgs.clangStdenv.mkDerivation {
          pname = "extern_traces_plugin";
          version = "1.0.0";

          src = ./packages/extern_traces_plugin;

          nativeBuildInputs = [
            pkgs.gnumake
            pkgs.git
            pkgs.coreutils
            pkgs.lld
          ];

          buildInputs = [
            ps-nix.packages.${system}.goldhen-sdk
            ps-nix.packages.${system}.toolchain
          ];

          preBuild = ''
            mkdir -p plugin_support/target/x86_64-unknown-freebsd/release
            cp ${pluginSupportProject.rootCrate.build.lib}/lib/libplugin_support-*.a \
               plugin_support/target/x86_64-unknown-freebsd/release/libplugin_support.a
          '';

          installPhase = ''
            runHook preInstall

            mkdir -p $out
            cp -r target $out

            runHook postInstall
          '';

          meta = {
            description = "PlayStation 4 extern traces plugin";
            license = pkgs.lib.licenses.mit;
            platforms = [ "x86_64-linux" "aarch64-darwin" ];
          };
        };

        treefmtConfig = {
          projectRootFile = "flake.nix";
          programs = {
            nixfmt.enable = true;
            rustfmt.enable = true;
            black.enable = true;
          };
        };
        treefmtEval = treefmt-nix.lib.evalModule pkgs treefmtConfig;

      in
      {
        packages = {
          extern_traces_viewer = cargoProject.workspaceMembers.extern_traces_viewer.build;
          extern_traces_plugin = extern_traces_plugin;
        };

        formatter = treefmtEval.config.build.wrapper;

        devShells.default = pkgs.mkShell {
          buildInputs = [
            (rustToolchainForPkgs pkgs)
            pkgs.crate2nix
          ];
        };
      }
    );
}
