{
  inputs,
  lib,
  ...
}: let
  inherit (inputs.self.lib) buildRustApp buildNpmApp mkRustDocker mkNpmDocker;
in {
  perSystem = {
    config,
    self',
    inputs',
    pkgs,
    system,
    ...
  }: let
    oxalateApps = {
      harvester = buildRustApp {inherit pkgs lib;} "oxalate_harvester";
      outlet = buildRustApp {inherit pkgs lib;} "oxalate_outlet";
      indexer = buildRustApp {inherit pkgs lib;} "oxalate_indexer";
      admin-ui =
        buildNpmApp {
          inherit pkgs lib;
          depsHash = "sha256-ahuZgG0Gp8vb2QYHaQmZpubf2je3FHJZ5wHo4CNV7e4=";
        } "admin_ui"
        ./../src/bins/admin_ui;
      frontend =
        buildNpmApp {
          inherit pkgs lib;
          depsHash = "sha256-uz323AChaROF9LFREnxHeS697owa/mjSjzroVZ1S47s=";
        } "frontend"
        ./../src/bins/frontend;
    };

    images = {
      harvester = mkRustDocker pkgs "oxalate-harvester-server" "oxalate_harvester" oxalateApps.harvester ["6767" "6969"];
      outlet = mkRustDocker pkgs "oxalate-outlet-server" "oxalate_outlet" oxalateApps.outlet [];
      indexer = mkRustDocker pkgs "oxalate-indexer-server" "oxalate_indexer" oxalateApps.indexer ["22267"];
      admin-ui = mkNpmDocker pkgs "oxalate-admin-ui-server" oxalateApps.admin-ui "3000";
      frontend = mkNpmDocker pkgs "oxalate-frontend-server" oxalateApps.frontend "4000";
    };

    servoPkgs = inputs'.servo.packages;
  in {
    packages = {
      harvester-app = oxalateApps.harvester;
      outlet-app = oxalateApps.outlet;
      indexer-app = oxalateApps.indexer;
      admin-ui-app = oxalateApps.admin-ui;
      frontend-app = oxalateApps.frontend;

      harvester-image = images.harvester;
      outlet-image = images.outlet;
      indexer-image = images.indexer;
      admin-ui-image = images.admin-ui;
      frontend-image = images.frontend;

      servo-app = servoPkgs.servo-app;
      servo-image = servoPkgs.servo-image;
      auth-app = servoPkgs.auth-app;
      auth-image = servoPkgs.auth-image;

      default = oxalateApps.indexer;
    };

    apps = {
      harvester = {
        type = "app";
        program = lib.getExe oxalateApps.harvester;
      };
      outlet = {
        type = "app";
        program = lib.getExe oxalateApps.outlet;
      };
      indexer = {
        type = "app";
        program = lib.getExe oxalateApps.indexer;
      };
      admin-ui = {
        type = "app";
        program = lib.getExe oxalateApps.admin-ui;
      };
      frontend = {
        type = "app";
        program = lib.getExe oxalateApps.frontend;
      };
      servo = {
        type = "app";
        program = lib.getExe servoPkgs.servo-app;
      };
      auth = {
        type = "app";
        program = lib.getExe servoPkgs.auth-app;
      };
    };
  };
}
