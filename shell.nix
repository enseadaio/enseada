with import <nixpkgs> {};
stdenv.mkDerivation {
    name = "enseada_shell";
    buildInputs = with pkgs; [
      pkgconfig
      openssl.dev
      yarn
      unstable.nodejs-14_x
    ];
}