{
  pkgs,
  lib,
  inputs,
  kubenix,
  ...
}: {
  flake.kubenixModules.pods = {
    kubenix,
    pkgs,
    config,
    self',
    ...
  }: {
    kubernetes.resources.configMaps.servo-config.data = {
      "Servo.toml" = builtins.readFile ./../../deploy/Servo.toml;
    };

    docker = {
      images.indexer.image = self'.packages.indexer-image;
      images.harvester.image = self'.packages.harvester-image;
      images.outlet.image = self'.packages.outlet-image;
      images.auth.image = self'.packages.auth-image;
      images.servo.image = self'.packages.servo-image;
      images.frontend.image = self'.packages.frontend-image;
      images.admin-ui.image = self'.packages.admin-ui-image;
    };

    kubernetes.resources.pods = let
      images = config.docker.images;
    in {
      indexer.spec.containers.indexer.image = images.indexer.path;
      harvester.spec.containers.harvester.image = images.harvester.path;
      outlet.spec.containers.outlet.image = images.outlet.path;
      auth.spec.containers.auth.image = images.auth.path;
      servo.spec = {
        containers.servo = {
          image = images.servo.path;
          volumeMounts = [
            {
              name = "config-volume";
              mountPath = "/etc/servo";
              readOnly = true;
            }
          ];
          args = ["--config" "/etc/servo/Servo.toml"];
        };
        volumes = [
          {
            name = "config-volume";
            configMap.name = "servo-config";
          }
        ];
      };
      frontend.spec.containers.frontend.image = images.frontend.path;
      admin-ui.spec.containers.admin-ui.image = images.admin-ui.path;

      postgres.spec.containers.db.image = "postgres:16";
      neo4j.spec.containers.db.image = "neo4j:latest";
      paradedb.spec.containers.db.image = "paradedb/paradedb:latest";
      redpanda.spec.containers.broker.image = "docker.redpanda.com/redpandadata/redpanda:latest";
    };

    kubernetes.resources.services = {
      postgres.spec = {
        selector.app = "postgres";
        ports = [
          {
            port = 5432;
            protocol = "TCP";
            name = "postgres";
          }
        ];
      };
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
      redpanda.spec = {
        selector.app = "redpanda";
        ports = [
          {
            port = 9092;
            protocol = "TCP";
            name = "kafka";
          }
        ];
      };
    };
  };
}
