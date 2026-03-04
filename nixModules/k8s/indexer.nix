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
              env = [
                {
                  name = "RUST_LOG";
                  value = "info";
                }
                {
                  name = "KAFKA_PORT";
                  value = "19092";
                }
                {
                  name = "KAFKA_DNS";
                  value = "redpanda";
                }
                {
                  name = "KAFKA_MESSAGE_TIMEOUT_MS";
                  value = "5000";
                }
                {
                  name = "KAFKA_INDEXER_LOGS_TOPIC";
                  value = "indexer_logs";
                }
                {
                  name = "KAFKA_OUTLET_LOGS_TOPIC";
                  value = "outlet_logs";
                }
                {
                  name = "NEO4J_PORT";
                  value = "7687";
                }
                {
                  name = "NEO4J_DNS";
                  value = "neo4j";
                }
                {
                  name = "DB_DNS";
                  value = "paradedb";
                }
                {
                  name = "DB_PORT";
                  value = "5432";
                }
                {
                  name = "POOL_MAX_CONN";
                  value = "25";
                }
                {
                  name = "INDEXER_BIND_ADDRESS";
                  value = "0.0.0.0";
                }
                {
                  name = "INDEXER_PORT";
                  value = "22267";
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
                  name = "NEO4J_AUTH";
                  valueFrom.secretKeyRef = {
                    name = "db-creds";
                    key = "neo4j_auth";
                  };
                }
              ];

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
