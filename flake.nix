{
  description = "Oxalate dev flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    pkgs = nixpkgs.legacyPackages."x86_64-linux";
  in {
    devShells."x86_64-linux".default = pkgs.mkShell {
      nativeBuildInputs = with pkgs; [
        pkg-config
        bun
        cmake
        gcc
        perl
        automake
        fish
        sqlx-cli
      ];

      buildInputs = with pkgs; [
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
        libx11
        libxext
        libxinerama
        libxcursor
        libxrender
        libxfixes
        libxi
        libxtst
        libclang
        llvm
      ];

      shellHook = ''
        export RUST_SRC_PATH="${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}"
        export SQLX_OFFLINE="true"

        export LIBCLANG_PATH="${pkgs.libclang.lib}/lib"
        export LD_LIBRARY_PATH="${pkgs.libclang.lib}/lib:${pkgs.llvm.lib}/lib''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

        if [ -f .env.dev ]; then
          export $(echo $(cat .env.dev | sed 's/#.*//g' | xargs) | envsubst)
          echo "✅ Local .env.dev loaded into shell"
        fi

        if [[ $(ps -p $PPID -o comm=) != "fish" ]]; then
          exec fish
        fi

      '';
    };
  };
}
