{ pkgs ? import <nixpkgs> { } }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "serve";
  version = "0.4.0";
  src = ./.;

  cargoLock = { lockFile = ./Cargo.lock; };
}
