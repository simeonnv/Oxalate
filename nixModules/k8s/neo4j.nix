{
  pkgs,
  lib,
  inputs,
  kubenix,
  ...
}: {
  flake.kubenixModules.neo4j = {
    kubenix,
    pkgs,
    config,
    self',
    ...
  }: {
    kubernetes.resources.statefulSets = {
      neo4j.spec = {
        volumeClaimTemplates = [
          {
            metadata.name = "neo4j-data";
            spec = {
              accessModes = ["ReadWriteOnce"];
              resources.requests.storage = "10Gi";
            };
          }
        ];

        replicas = 1;
        selector.matchLabels.app = "neo4j";
        template = {
          metadata.labels.app = "neo4j";
          spec = {
            containers.neo4j = {
              # ENV = {}; # TODO
              env = [
                {
                  name = "NEO4J_server_config_strict__validation_enabled";
                  value = "false";
                }
                {
                  name = "NEO4J_PLUGINS";
                  value = ''["apoc"]'';
                }
                {
                  name = "NEO4J_AUTH";
                  valueFrom.secretKeyRef = {
                    name = "db-creds";
                    key = "neo4j_auth";
                  };
                }
              ];

              image = "neo4j:latest";
              ports = [
                {
                  containerPort = 7474;
                  protocol = "TCP";
                  name = "http";
                }
                {
                  containerPort = 7687;
                  protocol = "TCP";
                  name = "bolt";
                }
              ];
              volumeMounts = [
                {
                  name = "neo4j-data";
                  mountPath = "/data";
                }
              ];
            };
          };
        };
      };
    };

    kubernetes.resources.services = {
      neo4j.spec = {
        selector.app = "neo4j";
        ports = [
          {
            port = 7474;
            protocol = "TCP";
            name = "http";
          }
          {
            port = 7687;
            protocol = "TCP";
            name = "bolt";
          }
        ];
      };
    };
  };
}
