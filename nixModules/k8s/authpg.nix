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
