let
  pkgs = import <nixpkgs> { };
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    just
    lld
    nodejs-16_x
  ];
}
