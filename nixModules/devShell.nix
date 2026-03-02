{
  pkgs,
  lib,
  ...
}: {
  perSystem = {
    config,
    pkgs,
    system,
    ...
  }: {
    devShells.default = pkgs.mkShell {
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

      shellHook = ''
        export RUST_SRC_PATH="${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}"
        export SQLX_OFFLINE="true"
        export LIBCLANG_PATH="${pkgs.libclang.lib}/lib"
        export LD_LIBRARY_PATH="${pkgs.libclang.lib}/lib:${pkgs.llvm.lib}/lib''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

        echo "We are so nix brur"
      '';
    };
  };
}
