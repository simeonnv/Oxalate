{
  pkgs,
  lib,
  inputs,
  kubenix,
  ...
}: {
  flake.kubenixModules.authpg = {
    kubenix,
    pkgs,
    config,
    self',
    ...
  }: {
    kubernetes.resources.statefulSets = {
      authpg.spec = {
        volumeClaimTemplates = [
          {
            metadata.name = "authpg-data";
            spec = {
              accessModes = ["ReadWriteOnce"];
              resources.requests.storage = "10Gi";
            };
          }
        ];

        replicas = 1;
        selector.matchLabels.app = "authpg";
        template = {
          metadata.labels.app = "authpg";
          spec = {
            containers.authpg = {
              # ENV = {}; # TODO
              env = [
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
                  name = "POSTGRES_DB";
                  value = "Oxalate";
                }
              ];

              image = "postgres:16";
              ports = [
                {
                  containerPort = 5432;
                  protocol = "TCP";
                }
              ];
              volumeMounts = [
                {
                  name = "authpg-data";
                  mountPath = "/var/lib/postgresql/data";
                }
              ];
            };
          };
        };
      };
    };

    kubernetes.resources.services = {
      authpg.spec = {
        selector.app = "authpg";
        ports = [
          {
            port = 15432;
            protocol = "TCP";
          }
        ];
      };
    };
  };
}
