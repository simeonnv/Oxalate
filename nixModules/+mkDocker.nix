{
  inputs,
  lib,
  ...
}: {
  flake.lib.mkDocker = pkgs: imageName: binName: binPkg:
    pkgs.dockerTools.buildLayeredImage {
      name = imageName;
      tag = "latest";
      contents = with pkgs; [cacert openssl boringssl];
      config.Cmd = ["${binPkg}/bin/${binName}"];
    };
}
