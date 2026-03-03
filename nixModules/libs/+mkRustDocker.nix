{
  inputs,
  lib,
  ...
}: {
  flake.lib.mkRustDocker = pkgs: imageName: binName: binPkg: exposedPorts:
    pkgs.dockerTools.buildLayeredImage {
      name = imageName;
      tag = "latest";
      contents = with pkgs; [cacert openssl boringssl];
      config = {
        Cmd = ["${binPkg}/bin/${binName}"];
        ExposedPorts = lib.genAttrs (map (p: "${p}/tcp") exposedPorts) (_: {});
      };
    };
}
