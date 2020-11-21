with import <nixpkgs> {};
stdenv.mkDerivation {
    name = "enseada_shell";
    buildInputs = with pkgs; [
      pkgconfig
      openssl.dev
      yarn
      nodejs-14_x
      go
    ];
}
