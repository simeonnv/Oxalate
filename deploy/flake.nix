{
  description = "Oxalate monorepo with multiple Rust Docker images";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      nixpkgsFor = forAllSystems (system: import nixpkgs { inherit system; });
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = nixpkgsFor.${system};
          buildApp = name: pkgs.rustPlatform.buildRustPackage {
            pname = name;
            version = "0.1.0";
            src = ../.;

            cargoLock = {
              lockFile = ../Cargo.lock;
              outputHashes = {
                "kafka_writer_rs-0.1.0" = "sha256-oBoHw1N20tq2wPiov6UOjlQrALJnOUuc//frIziEhqw=";
                "log_json_serializer-0.1.0" = "sha256-wwes1SkPhPmP9f4A18gpubrfhwhZ0nGbAZyrGTboiV8=";
              };
            };

            nativeBuildInputs = [ 
              pkgs.pkg-config
              pkgs.cmake        
              pkgs.perl
              pkgs.gcc
              pkgs.automake
            ];

            buildInputs = [ 
              pkgs.openssl
              pkgs.zstd         
              pkgs.lz4
              pkgs.curl
              pkgs.zlib
              pkgs.cmake
              
              pkgs.libx11
              pkgs.libxext
              pkgs.libxinerama
              pkgs.libxcursor
              pkgs.libxrender
              pkgs.libxfixes
              pkgs.libxi
              pkgs.libxtst 
            ];

            SQLX_OFFLINE_DIR = "../.sqlx";
            env = {
              SQLX_OFFLINE = "true";
            };
                                                
            preBuild = ''
              export SET_MAKE_JOBS=$NIX_BUILD_CORES
            '';            

            buildAndCheckFeatures = [ "--package" name ];
          };

          mkDocker = name: bin: pkgs.dockerTools.buildLayeredImage {
            name = name;
            tag = "latest";
            contents = [ pkgs.cacert pkgs.openssl ];
            config = {
              Cmd = [ "${bin}/bin/${name}" ];
              # Env = [ "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" ];
            };
          };

          harvester-bin = buildApp "harvester";
          outlet-bin = buildApp "outlet";
          indexer-bin = buildApp "indexer";
        in
        {
          # To build the binaries: nix build .#api-server
          harvester-server = harvester-bin; 
          outlet-server = outlet-bin;
          indexer-server = indexer-bin;

          # To build the images: nix build .#docker-api
          docker-harvester = mkDocker "oxalate-harvester-server" harvester-bin;
          docker-outlet = mkDocker "oxalate-outlet-server" outlet-bin;
          docker-indexer= mkDocker "oxalate-indexer-server" indexer-bin;
        });
    };
}
