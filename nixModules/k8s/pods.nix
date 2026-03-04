{
  pkgs,
  lib,
  inputs,
  kubenix,
  ...
}: {
  flake.kubenixModules.pods = {
    kubenix,
    pkgs,
    config,
    self',
    ...
  }: {
    kubernetes.resources.pods = let
      images = config.docker.images;
    in {
      harvester.spec.containers.harvester.image = images.harvester.path;

      outlet.spec.containers.outlet.image = images.outlet.path;

      auth.spec.containers.auth.image = images.auth.path;

      frontend.spec.containers.frontend.image = images.frontend.path;

      admin-ui.spec.containers.admin-ui.image = images.admin-ui.path;
    };
  };
}
