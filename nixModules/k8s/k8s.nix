{
  pkgs,
  lib,
  inputs,
  kubenix,
  self,
  ...
}: {
  perSystem = {
    config,
    pkgs,
    system,
    self',
    ...
  }: let
    kubenixEval = inputs.kubenix.evalModules.${system} {
      module = {kubenix, ...}: {
        kubenix.project = "Oxalate";
        imports = with kubenix.modules;
          [k8s docker]
          ++ (with self.kubenixModules; [
            pods
            docker
            paradedb
            servo
            neo4j
            authpg
            redpanda
            indexer
          ]);

        kubernetes.resources.secrets.db-creds.stringData = {
          postgres_user = "ref+sops://sops_secrets.yaml?key=env.postgres_user";
          postgres_password = "ref+sops://sops_secrets.yaml?key=env.postgres_password";
          neo4j_auth = "ref+sops://sops_secrets.yaml?key=env.neo4j_auth";
        };

        _module.args = rec {
          envArgs = {
            SQLX_OFFLINE = false;
            RUST_LOG = "info";

            PUBLIC_HARVESTER_BIND_ADDRESS = "0.0.0.0";
            PUBLIC_HARVESTER_PORT = 6767;

            PRIVATE_HARVESTER_BIND_ADDRESS = "0.0.0.0";
            PRIVATE_HARVESTER_PORT = 6969;

            HARVESTER_DNS = "0.0.0.0";

            INDEXER_BIND_ADDRESS = "0.0.0.0";
            INDEXER_PORT = 22267;
            INDEXER_DNS = "0.0.0.0";

            POSTGRES_USER.secretKeyRef = {
              name = "db-creds";
              key = "postgres_user";
            };

            POSTGRES_PASSWORD.secretKeyRef = {
              name = "db-creds";
              key = "postgres_password";
            };

            POSTGRES_DB = "Oxalate";
            DB_BIND_ADDRESS = "0.0.0.0";
            DB_DNS = "0.0.0.0";
            DB_PORT = 6666;
            POOL_MAX_CONN = 25;

            KAFKA_BIND_ADDRESS = "0.0.0.0";

            KAFKA_PORT = 19092;
            KAFKA_DNS = "0.0.0.0";
            KAFKA_MESSAGE_TIMEOUT_MS = 5000;

            NEO4J_AUTH.secretKeyRef = {
              name = "db-creds";
              key = "neo4j_auth";
            };

            NEO4J_BIND_ADDRESS = "0.0.0.0";
            NEO4J_PORT = 7687;
            NEO4J_DNS = "0.0.0.0";

            PARSER_BIND_ADDRESS = "0.0.0.0";
            PARSER_PORT = 11167;
            PARSER_DNS = "0.0.0.0";

            KAFKA_HARVESTER_LOGS_TOPIC = "harvester_logs";
            KAFKA_INDEXER_LOGS_TOPIC = "indexer_logs";
            KAFKA_OUTLET_LOGS_TOPIC = "outlet_logs";

            URLS_FILE = "./urls.txt";
          };

          mkEnv = name: let
            value = envArgs.${name};
          in
            {
              name = name;
            }
            // (
              if builtins.isAttrs value && value ? secretKeyRef
              then {
                valueFrom.secretKeyRef = value.secretKeyRef;
              }
              else {
                value =
                  if builtins.isBool value
                  then lib.boolToString value
                  else toString value;
              }
            );
        };

        docker.registry.url = "localhost:5000";
      };
      specialArgs = {inherit self';};
    };
  in {
    packages = {
      k8s = kubenixEval.config.kubernetes.result;

      deploy = pkgs.writeShellScriptBin "deploy" ''
        set -e

        echo "Decrypting secrets with vals and applying to Kubernetes..."

        cat ${kubenixEval.config.kubernetes.result} | \
        ${pkgs.vals}/bin/vals eval -f - | \
        ${pkgs.kubectl}/bin/kubectl apply -f -

        echo "Done"
      '';
    };
  };
}
