{
  pkgs,
  lib,
  inputs,
  kubenix,
  ...
}: {
  flake.kubenixModules.redpanda = {
    kubenix,
    pkgs,
    config,
    self',
    ...
  }: {
    kubernetes.resources.statefulSets = {
      redpanda.spec = {
        volumeClaimTemplates = [
          {
            metadata.name = "redpanda-data";
            spec = {
              accessModes = ["ReadWriteOnce"];
              resources.requests.storage = "10Gi";
            };
          }
        ];

        replicas = 1;
        selector.matchLabels.app = "redpanda";
        template = {
          metadata.labels.app = "redpanda";
          spec = {
            containers.redpanda = {
              image = "docker.redpanda.com/redpandadata/redpanda:v23.3.10";
              args = [
                "redpanda"
                "start"
                "--overprovisioned"
                "--smp 1"
                "--node-id 0"

                "--kafka-addr internal://0.0.0.0:9092,external://0.0.0.0:19092"
                "--advertise-kafka-addr internal://redpanda:9092,external://localhost:19092"
                "--pandaproxy-addr internal://0.0.0.0:8082,external://0.0.0.0:18082"
                "--advertise-pandaproxy-addr internal://redpanda:8082,external://localhost:18082"

                "--memory 1G"
                "--reserve-memory 0M"
                "--check=false"
              ];
              ports = [
                {
                  containerPort = 19092;
                  protocol = "TCP";
                  name = "kafka";
                }
                {
                  containerPort = 18082;
                  protocol = "TCP";
                  name = "redpanda";
                }
              ];
              volumeMounts = [
                {
                  name = "redpanda-data";
                  mountPath = "/var/lib/redpanda/data";
                }
              ];
            };
          };
        };
      };
    };

    kubernetes.resources.services = {
      redpanda.spec = {
        selector.app = "redpanda";
        ports = [
          {
            port = 19092;
            protocol = "TCP";
            name = "kafka";
          }
          {
            port = 18082;
            protocol = "TCP";
            name = "redpanda";
          }
        ];
      };
    };
  };
}
