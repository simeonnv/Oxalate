{
  pkgs,
  lib,
  inputs,
  kubenix,
  ...
}: {
  flake.kubenixModules.docker = {
    kubenix,
    pkgs,
    config,
    self',
    ...
  }: {
    docker = {
      images.indexer.image = self'.packages.indexer-image;
      images.harvester.image = self'.packages.harvester-image;
      images.outlet.image = self'.packages.outlet-image;
      images.auth.image = self'.packages.auth-image;
      images.servo.image = self'.packages.servo-image;
      images.frontend.image = self'.packages.frontend-image;
      images.admin-ui.image = self'.packages.admin-ui-image;
    };
  };
}
