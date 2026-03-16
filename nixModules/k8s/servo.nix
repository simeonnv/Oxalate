{
  pkgs,
  lib,
  inputs,
  kubenix,
  ...
}: {
  flake.kubenixModules.servo = {
    kubenix,
    pkgs,
    config,
    self',
    ...
  }: {
    kubernetes.resources.configMaps.servo-config.data = {
      "Servo.toml" = builtins.readFile ./../../deploy/Servo.toml;
    };

    kubernetes.resources.deployments.servo.spec = {
      replicas = 1;
      selector.matchLabels.app = "servo";
      template = {
        metadata.labels.app = "servo";
        spec = {
          containers.servo = {
            image = config.docker.images.servo.path;
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
      };
    };

    kubernetes.resources.services.servo = {
      metadata.name = "servo";
      spec = {
        selector.app = "servo";
        ports = [
          {
            name = "http";
            port = 80;
            targetPort = 8080;
          }
        ];
      };
    };

    kubernetes.resources.ingresses.main-ingress = {
      spec = {
        rules = [
          {
            host = "localhost";
            http.paths = [
              {
                path = "/";
                pathType = "Prefix";
                backend.service = {
                  name = "servo";
                  port.number = 80;
                };
              }
            ];
          }
        ];
      };
    };

    kubernetes.resources.deployments.redis.spec = {
      replicas = 1;

      selector.matchLabels = {
        app = "redis";
        tier = "cache";
      };

      template = {
        metadata.labels = {
          app = "redis";
          tier = "cache";
        };

        spec.containers.redis = {
          image = "redis:7-alpine";
          resources = {
            requests = {
              cpu = "100m";
              memory = "128Mi";
            };
            limits = {
              cpu = "500m";
              memory = "512Mi";
            };
          };
          ports = [
            {
              containerPort = 6379;
              name = "redis";
            }
          ];
        };
      };
    };

    kubernetes.resources.services.redis = {
      metadata.name = "redis";
      spec = {
        selector = {
          app = "redis";
          tier = "cache";
        };
        ports = [
          {
            port = 6379;
            targetPort = 6379;
            name = "redis";
          }
        ];
      };
    };
  };
}
