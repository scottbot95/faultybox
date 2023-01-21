{ pkgs,
  faultybox,
  craneLib,
  advisory-db,
  ...
}:
let
  inherit (pkgs) lib system;
  inherit (faultybox) cargoArtifacts src buildInputs;
in {
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
}