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
    kubernetes.resources.deployments = {
      indexer.spec = {
        replicas = 2;
        selector.matchLabels.app = "indexer";
        template = {
          metadata.labels.app = "indexer";
          spec = {
            containers.indexer = {
              # ENV = {}; # TODO
              image = config.docker.images.indexer.path;
              ports = [
                {
                  containerPort = 22267;
                  protocol = "TCP";
                }
              ];
            };
          };
        };
      };
    };

    kubernetes.resources.services = {
      indexer.spec = {
        selector.app = "indexer";
        ports = [
          {
            port = 22267;
            protocol = "TCP";
          }
        ];
      };
    };
  };
}
