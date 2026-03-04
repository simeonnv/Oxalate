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

        docker.registry.url = "localhost:5000";
      };
      specialArgs = {inherit self';};
    };
  in {
    packages = {
      k8s = kubenixEval.config.kubernetes.result;

      push-images = kubenixEval.config.docker.copyScript;

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
