{
  pkgs,
  lib,
  inputs,
  kubenix,
  ...
}: {
  flake.kubenixModules.auth = {
    kubenix,
    pkgs,
    config,
    self',
    ...
  }: {
    kubernetes.resources.statefulSets = {
      auth.spec = {
        replicas = 1;
        selector.matchLabels.app = "auth";
        template = {
          metadata.labels.app = "auth";
          spec = {
            containers.auth = {
              image = config.docker.images.auth.path;
              ports = [
                {
                  containerPort = 8989;
                  protocol = "TCP";
                }
              ];
              env = [
                {
                  name = "EXAMPLE";
                  value = "VALUE";
                }
                {
                  name = "EXAMPLESECRET";
                  valueFrom.secretKeyRef = {
                    name = "db-creds";
                    key = "postgres_password";
                  };
                }
              ];
            };
          };
        };
      };
    };
    kubernetes.resources.services = {
      auth.spec = {
        selector.app = "auth";
        ports = [
          {
            port = 8989;
            protocol = "TCP";
          }
        ];
      };
    };
  };
}
