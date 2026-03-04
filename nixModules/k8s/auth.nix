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
                  name = "RUST_LOG";
                  value = "info";
                }
                {
                  name = "AUTH_ADDRESS";
                  value = "0.0.0.0";
                }
                {
                  name = "AUTH_PORT";
                  value = "8989";
                }
                {
                  name = "POSTGRES_DB";
                  value = "Oxalate";
                }
                {
                  name = "POSTGRES_USER";
                  valueFrom.secretKeyRef = {
                    name = "db-creds";
                    key = "postgres_user";
                  };
                }
                {
                  name = "POSTGRES_PASSWORD";
                  valueFrom.secretKeyRef = {
                    name = "db-creds";
                    key = "postgres_password";
                  };
                }
                {
                  name = "DB_DNS";
                  value = "authpg";
                }
                {
                  name = "DB_PORT";
                  value = "15432";
                }
                {
                  name = "POOL_MAX_CONN";
                  value = "5";
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
