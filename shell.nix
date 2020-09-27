let
  moz_overlay = import (builtins.fetchTarball {
    url = https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz;
    sha256 = "11wqrg86g3qva67vnk81ynvqyfj0zxk83cbrf0p9hsvxiwxs8469";
  });
  nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
in
  with nixpkgs;
  stdenv.mkDerivation {
    name = "enseada_shell";
    buildInputs = with nixpkgs; [
      pkgconfig
      openssl.dev
      (rustChannelOf { rustToolchain = ./rust-toolchain; }).rust
      yarn
      unstable.nodejs-14_x
    ];
  }