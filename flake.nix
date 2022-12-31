{
  description = "Gecko: an online game written in rust";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-22.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ 
          (import rust-overlay)
          (super: self: {

          })
        ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rust-toolchain = with pkgs; [
          (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
          cargo-make
          trunk
          wasm-bindgen-cli
        ];
      in
      with pkgs;
      rec {
        packages = {
          faultybox-frontend = mkYarnModules {
            pname = "faultybox-frontend";
            version = "0.1.0";
            packageJSON = ./frontend/package.json;
            yarnLock = ./frontend/yarn.lock;
            yarnNix = ./frontend/yarn.nix;
          };

          faultybox = rustPlatform.buildRustPackage rec {
            pname = "faultybox";
            version = "0.1.0";

            nativeBuildInputs = rust-toolchain ++ [
              makeWrapper
              sass
            ];

            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            buildPhase = ''
              runHook preBuild

              mkdir -p ./frontend/node_modules
              cp -r ${packages.faultybox-frontend}/node_modules/\@patternfly ./frontend/node_modules
              PATH=${packages.faultybox-frontend}/node_modules/.bin:$PATH cargo make build -j $NIX_BUILD_CORES --release

              runHook postBuild
            '';

            installPhase = ''
              runHook preInstall

              # mkdir -p $out
              # cp -r ./ $out/

              mkdir -p $out/bin
              
              cp -v ./target/release/server $out/bin/
              cp -rv ./dist/ $out/

              wrapProgram $out/bin/server --add-flags "--static-dir $out/dist/"

              runHook postInstall
            '';

            checkPase = ''
              runHook preCheck
              runHook postCheck
            '';
          };
        };
        defaultPackage = packages.faultybox;
        devShells.default = mkShell {
          buildInputs = rust-toolchain ++ [
            yarn2nix
          ];
        };
      }
    );
}