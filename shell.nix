# Fallback dev toolchain for building/running the Rust API + importer on NixOS.
#
# Prefer the host toolchain: install `cargo`, `rustc`, and `gcc` from nixpkgs (system config
# or home-manager) and this file is unnecessary — `make load-porsche` / `cargo build` work
# directly. It's kept only for hosts without those on PATH. The build needs nothing else:
# TLS is rustls+ring (no openssl/pkg-config) and mimalloc/sqlite compile via `cc`.
{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
  packages = with pkgs; [ cargo rustc gcc ];
}
