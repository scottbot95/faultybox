{ pkgs,
  craneLib,
  ...
}:
let
  inherit (pkgs) lib buildNpmPackage;
  inherit (lib) cleanSourceWith optionals hasSuffix hasInfix;

  node_modules = buildNpmPackage {
    pname = "faultybox-frontend-assets";
    version = "0.1.0";
    src = lib.cleanSourceWith {
      src = ../frontend;
      filter = path: _type:
        (builtins.match ".*/frontend/package.*json" path) != null;
    };

    dontNpmBuild = true;

    npmDepsHash = "sha256-auK6NUGjdQyGTawrITh2+XUymLZP77PraE5IYnR8hOM=";
    # npmDepsHash = lib.fakeHash;
  };

  src = cleanSourceWith {
    src = ../.;
    filter = path: type:
      (hasSuffix "\.html" path) ||
      (hasSuffix "\.scss" path) ||
      (hasSuffix "\.ron" path) ||
      (hasInfix "/assets/" path) ||
      (craneLib.filterCargoSources path type);
  };

  commonArgs = {
    inherit src;
    strictDeps = true;

    buildInputs = [
      # Add additional build inputs here
    ] ++ optionals pkgs.stdenv.isDarwin [
      # Additional darwin specific inputs can be set here
      pkgs.libiconv
    ];
  };

  nativeArgs = commonArgs // {

  };

  # Build *just* the cargo dependencies, so we can reuse
  # all of that work (e.g. via cachix) when running in CI
  cargoArtifacts = craneLib.buildDepsOnly nativeArgs;

  wasmArgs = commonArgs // {
    pname = "faultybox-wasm";
    cargoExtraArgs = "--package=frontend";
    CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
  };

  cargoArtifactsWasm = craneLib.buildDepsOnly wasmArgs;

  frontend = craneLib.buildTrunkPackage (wasmArgs // {
    pname = "faultybox-frontend";
    cargoArtifacts = cargoArtifactsWasm;
    trunkExtraBuildArgs = "--public-url /assets/";

    wasm-bindgen-cli = pkgs.wasm-bindgen-cli_0_2_100;

    # Trunk expects the current directory to be the crate to compile.
    # Also pull in node_modules
    preBuild = ''
      cd ./frontend

      mkdir -p ./node_modules
      cp -r ${node_modules}/lib/node_modules/faultybox/node_modules/* ./node_modules/
    '';

    postBuild = ''
      cd ..
    '';
  });

  faultybox = craneLib.buildPackage (nativeArgs // {
    inherit cargoArtifacts;

    FRONTEND_DIST = frontend;
  });
in {
  inherit faultybox node_modules;

  default = faultybox;
}
