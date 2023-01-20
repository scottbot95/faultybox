{
  description = "Build a cargo project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, advisory-db, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        inherit (pkgs) lib;

        node_modules = pkgs.buildNpmPackage rec {
          pname = "faultybox-frontend";
          version = "0.1.0";
          src = lib.cleanSourceWith {
            src = ./frontend;
            filter = path: _type:
              (builtins.match ".*/frontend/package.*json" path) != null;
          };

          dontNpmBuild = true;

          npmDepsHash = "sha256-LMGggPsm3AgKRscSC1sl2dLi8TT0dHW4VmmU4HB/x/Y=";
        };

        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
        src = let
          customFilter = path: _type: builtins.match ".*(ron|html|scss)$" path != null;
          customOrCargo = path: type:
            (customFilter path type) || (craneLib.filterCargoSources path type);
        in lib.cleanSourceWith {
          src = ./.;
          filter = customOrCargo;
        };

        nativeBuildInputs = with pkgs; [
          makeWrapper
          cargo-make
          trunk
          wasm-bindgen-cli
        ];

        buildInputs = [
          # Add additional build inputs here
        ] ++ lib.optionals pkgs.stdenv.isDarwin [
          # Additional darwin specific inputs can be set here
          pkgs.libiconv
        ];

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly {
          inherit src buildInputs;
        };

        faultybox-frontend = craneLib.buildPackage {
          inherit cargoArtifacts src buildInputs nativeBuildInputs;

          postPatch = ''
            cp -r ${node_modules}/lib/node_modules/faultybox/node_modules/\@patternfly ./frontend/node_modules
          '';

          buildPhaseCargoCommand = ''
            export PATH=${node_modules}/lib/node_modules/faultybox/node_modules/.bin:$PATH 
            cargo make build -j $NIX_BUILD_CORES --release
          '';

          installPhaseCommand = ''
            mkdir -p $out/bin
            
            cp -v ./target/release/server $out/bin/
            cp -rv ./dist/ $out/

            wrapProgram $out/bin/server --add-flags "--static-dir $out/dist/"
          '';
        };

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
        faultybox = craneLib.buildPackage {
          inherit cargoArtifacts src buildInputs nativeBuildInputs;

          # cargoBuildCommand = "cargo make build -j $NIX_BUILD_CORES --release";
          cargoExtraArgs = "--bin server";
        };
      in
      {
        checks = {
          # Build the crate as part of `nix flake check` for convenience
          inherit faultybox;

          # Run clippy (and deny all warnings) on the crate source,
          # again, resuing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          faultybox-clippy = craneLib.cargoClippy {
            inherit cargoArtifacts src buildInputs;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          };

          faultybox-doc = craneLib.cargoDoc {
            inherit cargoArtifacts src buildInputs;
          };

          # Check formatting
          faultybox-fmt = craneLib.cargoFmt {
            inherit src;
          };

          # Audit dependencies
          faultybox-audit = craneLib.cargoAudit {
            inherit src advisory-db;
          };

          # Run tests with cargo-nextest
          # Consider setting `doCheck = false` on `faultybox` if you do not want
          # the tests to run twice
          faultybox-nextest = craneLib.cargoNextest {
            inherit cargoArtifacts src buildInputs;
            partitions = 1;
            partitionType = "count";
          };
        } // lib.optionalAttrs (system == "x86_64-linux") {
          # NB: cargo-tarpaulin only supports x86_64 systems
          # Check code coverage (note: this will not upload coverage anywhere)
          faultybox-coverage = craneLib.cargoTarpaulin {
            inherit cargoArtifacts src;
          };
        };

        packages.default = faultybox;
        packages.frontend = faultybox-frontend;

        apps.default = flake-utils.lib.mkApp {
          drv = faultybox;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = builtins.attrValues self.checks;

          # Extra inputs can be added here
          inherit nativeBuildInputs;
        };
      });
}