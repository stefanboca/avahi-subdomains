{
  lib,
  rustPlatform,
  ...
}:
rustPlatform.buildRustPackage {
  pname = "avahi-subdomains";
  inherit ((lib.importTOML ../Cargo.toml).package) version;

  src = with lib.fileset;
    toSource {
      root = ../.;
      fileset = unions [
        (fileFilter (file: file.hasExt "rs") ../.)
        ../Cargo.toml
        ../Cargo.lock
      ];
    };

  cargoLock.lockFile = ../Cargo.lock;

  meta = {
    mainProgram = "avahi-subdomains";
  };
}
