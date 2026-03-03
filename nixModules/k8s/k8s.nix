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
  }: {
    packages.k8s =
      (inputs.kubenix.evalModules.${system} {
        module = {kubenix, ...}: {
          imports = with kubenix.modules;
            [
              k8s
              docker
            ]
            ++ (with self.kubenixModules; [
              indexer
            ]);
          docker.registry.url = "localhost:5000";
        };
        specialArgs = {inherit self';};
      }).config.kubernetes.result;
  };
}
