{ pkgs,
  craneLib,
  ...
}:
let
  inherit (pkgs) lib buildNpmPackage;

  node_modules = buildNpmPackage rec {
    pname = "faultybox-frontend-assets";
    version = "0.1.0";
    src = lib.cleanSourceWith {
      src = ../frontend;
      filter = path: _type:
        (builtins.match ".*/frontend/package.*json" path) != null;
    };

    dontNpmBuild = true;

    npmDepsHash = "sha256-auK6NUGjdQyGTawrITh2+XUymLZP77PraE5IYnR8hOM=";
#    npmDepsHash = lib.fakeHash;
  };

  src = let
    customFilter = path: _type: builtins.match "(.*/frontend/assets/.*|.*(ron|html|scss))$" path != null;
    customOrCargo = path: type:
      (customFilter path type) || (craneLib.filterCargoSources path type);
  in lib.cleanSourceWith {
    src = ../.;
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

  # Build the actual crate itself, reusing the dependency
  # artifacts from above.
  faultybox = craneLib.buildPackage {
    inherit cargoArtifacts src buildInputs nativeBuildInputs;

    postPatch = ''
      mkdir -p ./frontend/node_modules
      cp -r ${node_modules}/lib/node_modules/faultybox/node_modules/\@patternfly ./frontend/node_modules/
      cp -r ${node_modules}/lib/node_modules/faultybox/node_modules/\@fortawesome ./frontend/node_modules/
    '';

    buildPhaseCargoCommand = ''
      export PATH=${node_modules}/lib/node_modules/faultybox/node_modules/.bin:$PATH
      cargoBuildLog=$(mktemp cargoBuildLogXXXX.json)
      cargo make build -j $NIX_BUILD_CORES --release --message-format json-render-diagnostics | tee $cargoBuildLog
    '';

    installPhaseCommand = ''
      mkdir -p $out/bin
      
      cp -v ./target/release/server $out/bin/
      cp -rv ./dist/ $out/

      wrapProgram $out/bin/server --add-flags "--static-dir $out/dist/"
    '';
  };
in {
  inherit faultybox;

  default = faultybox;
}
