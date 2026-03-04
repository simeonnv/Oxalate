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

    kubernetes.resources.deployments = {
      servo.spec = {
        replicas = 2;
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
  };
}
