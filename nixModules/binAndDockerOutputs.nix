{
  inputs,
  lib,
  ...
}: let
  inherit (inputs.self.lib) buildRustBin mkDocker;
in {
  perSystem = {
    config,
    self',
    inputs',
    pkgs,
    system,
    ...
  }: let
    bins = {
      harvester = buildRustBin {inherit pkgs lib;} "oxalate_harvester";
      outlet = buildRustBin {inherit pkgs lib;} "oxalate_outlet";
      indexer = buildRustBin {inherit pkgs lib;} "oxalate_indexer";
    };

    images = {
      harvester = mkDocker pkgs "oxalate-harvester-server" "oxalate_harvester" bins.harvester;
      outlet = mkDocker pkgs "oxalate-outlet-server" "oxalate_outlet" bins.outlet;
      indexer = mkDocker pkgs "oxalate-indexer-server" "oxalate_indexer" bins.indexer;
    };
  in {
    packages = {
      harvester-bin = bins.harvester;
      outlet-bin = bins.outlet;
      indexer-bin = bins.indexer;

      harvester-image = images.harvester;
      outlet-image = images.outlet;
      indexer-image = images.indexer;

      default = bins.harvester;
    };

    apps = {
      harvester = {
        type = "app";
        program = lib.getExe bins.harvester;
      };
      outlet = {
        type = "app";
        program = lib.getExe bins.outlet;
      };
      indexer = {
        type = "app";
        program = lib.getExe bins.indexer;
      };
    };
  };
}
