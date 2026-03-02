{
  inputs,
  lib,
  ...
}: {
  flake.lib.getDeps = pkgs: lib: {
    nativeBuildInputs = with pkgs; [
      direnv
      pkg-config
      bun
      cmake
      gcc
      perl
      automake
      sqlx-cli
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
