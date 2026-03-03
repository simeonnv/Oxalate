{
  pkgs,
  lib,
  inputs,
  kubenix,
  ...
}: {
  flake.kubenixModules.indexer = {
    kubenix,
    pkgs,
    config,
    self',
    ...
  }: {
    docker = {
      images.indexer.image = self'.packages.indexer-image;
    };
    kubernetes.resources.pods.indexer = {
      spec.containers = {
        indexer.image = config.docker.images.indexer.path;
      };
    };
  };

  # perSystem = {
  #   config,
  #   pkgs,
  #   system,
  #   ...
  # }: {
  #   packages.k8s =
  #     (inputs.kubenix.evalModules.${system} {
  #       module = {kubenix, ...}: {
  #         imports = [kubenix.modules.k8s];
  #         kubernetes.resources.pods.example.spec.containers.nginx.image = "nginx";
  #       };
  #     }).config.kubernetes.result;
  # };
}
