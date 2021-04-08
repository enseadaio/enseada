let
  pkgs = import <nixpkgs> { };
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    pkgconfig
    openssl.dev
    yarn
    nodejs-14_x
    go
    protobuf
  ];

  shellHook = ''
  export PROTOBUF_LOCATION=${pkgs.protobuf}
  export PROTOC=$PROTOBUF_LOCATION/bin/protoc
  export PROTOC_INCLUDE=$PROTOBUF_LOCATION/include
  '';
}
