{
  description = "Oxalate monorepo with multiple Rust Docker images using Naersk";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    naersk.url = "github:nix-community/naersk"; 
  };

  outputs = { self, nixpkgs, naersk }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      nixpkgsFor = forAllSystems (system: import nixpkgs { inherit system; });
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = nixpkgsFor.${system};
          naersk-lib = pkgs.callPackage naersk { }; 

          buildApp = name: naersk-lib.buildPackage {
            pname = name;
            version = "0.1.0";
            src = ../.; 

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
              # Note: cmake removed here since it's already in nativeBuildInputs
              pkgs.libx11
              pkgs.libxext
              pkgs.libxinerama
              pkgs.libxcursor
              pkgs.libxrender
              pkgs.libxfixes
              pkgs.libxi
              pkgs.libxtst
            ];

            SQLX_OFFLINE = "true";
            SQLX_OFFLINE_DIR = "../.sqlx"; 

            preBuild = ''
              export SET_MAKE_JOBS=$NIX_BUILD_CORES
            '';

            cargoBuildOptions = x: x ++ [ "-p" name ];
            cargoTestOptions = x: x ++ [ "-p" name ];
          };

          mkDocker = imageName: binName: binPkg: pkgs.dockerTools.buildLayeredImage {
            name = imageName;
            tag = "latest";
            contents = [
              pkgs.cacert
              pkgs.openssl
            ];
            config = {
              Cmd = [ "${binPkg}/bin/${binName}" ];
            };
          };

          harvester-bin = buildApp "oxalate_harvester";
          outlet-bin = buildApp "oxalate_outlet";
          indexer-bin = buildApp "oxalate_indexer";
        in
        {
          harvester-server = harvester-bin;
          outlet-server = outlet-bin;
          indexer-server = indexer-bin;

          docker-harvester = mkDocker "oxalate-harvester-server" "oxalate_harvester" harvester-bin;
          docker-outlet = mkDocker "oxalate-outlet-server" "oxalate_outlet" outlet-bin;
          docker-indexer = mkDocker "oxalate-indexer-server" "oxalate_indexer" indexer-bin;
        });
    };
}
