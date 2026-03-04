{
  inputs,
  lib,
  ...
}: {
  flake.lib.getDeps = pkgs: lib: {
    nativeBuildInputs = with pkgs; [
      go
      ninja
      patch
      stdenv.cc
      nasm
      git
      clang

      direnv
      pkg-config
      nodejs_25
      cmake
      gcc
      perl
      automake
      sqlx-cli
      llvm
      cyrus_sasl
      libclang
    ];

    buildInputs = with pkgs;
      [
        cargo
        rustc
        rustfmt
        clippy
        rust-analyzer
        openssl
        zstd
        lz4
        zlib
        curl
        cyrus_sasl
        libclang
        llvm
      ]
      ++ lib.optionals pkgs.stdenv.isLinux [
        libX11
        libXext
        libXinerama
        libXcursor
        libXrender
        libXfixes
        libXi
        libXtst
      ];

    devTools = with pkgs; [
      direnv
      bun
      sqlx-cli
      cargo
      rustc
      rustfmt
      clippy
      rust-analyzer
      cyrus_sasl
      libclang
      llvm
    ];
  };
}
