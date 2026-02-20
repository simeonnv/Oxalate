{
  description = "Oxalate dev flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = { self, nixpkgs }: let
    pkgs = nixpkgs.legacyPackages."x86_64-linux";
  in {
    devShells."x86_64-linux".default = pkgs.mkShell {
      nativeBuildInputs = with pkgs; [ 
        pkg-config 
        cmake 
        gcc 
        perl 
        automake 
        fish
      ];

      buildInputs = with pkgs; [
        cargo rustc rustfmt clippy rust-analyzer
        openssl zstd lz4 zlib 
        curl              
        cyrus_sasl        
        libx11 libxext libxinerama libxcursor libxrender libxfixes libxi libxtst
      ];

      shellHook = ''
        export RUST_SRC_PATH="${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}"
        export SQLX_OFFLINE="true"

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
