{ pkgs ? import <nixpkgs> { } }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "serve";
  version = "0.6.1";
  src = ./.;

  cargoLock = { lockFile = ./Cargo.lock; };
}
