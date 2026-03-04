{
  pkgs,
  lib,
  inputs,
  kubenix,
  ...
}: {
  flake.kubenixModules.paradedb = {
    kubenix,
    pkgs,
    config,
    self',
    ...
  }: {
    kubernetes.resources.statefulSets = {
      paradedb.spec = {
        volumeClaimTemplates = [
          {
            metadata.name = "pardb-data";
            spec = {
              accessModes = ["ReadWriteOnce"];
              resources.requests.storage = "10Gi";
            };
          }
        ];

        replicas = 1;
        selector.matchLabels.app = "paradedb";
        template = {
          metadata.labels.app = "paradedb";
          spec = {
            containers.paradedb = {
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

              image = "paradedb/paradedb:latest";
              ports = [
                {
                  containerPort = 5432;
                  protocol = "TCP";
                }
              ];
              volumeMounts = [
                {
                  name = "pardb-data";
                  mountPath = "/var/lib/postgresql/data";
                }
              ];
            };
          };
        };
      };
    };

    kubernetes.resources.services = {
      paradedb.spec = {
        selector.app = "paradedb";
        ports = [
          {
            port = 5432;
            protocol = "TCP";
          }
        ];
      };
    };
  };
}
