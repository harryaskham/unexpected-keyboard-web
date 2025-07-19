{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.rustc
    pkgs.cargo
    pkgs.wasm-pack
    pkgs.lld
    pkgs.rust-analyzer
  ];

  RUSTFLAGS = "--cfg=web_sys_unstable_apis";
  CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "${pkgs.lld}/bin/lld";
}