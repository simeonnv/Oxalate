# Copyright 2026 Simeon Vangelov
# SPDX-License-Identifier: GPL-3.0-only

{
  inputs,
  lib,
  ...
}: {
  flake.lib.mkNpmDocker = pkgs: imageName: npmApp: port:
    pkgs.dockerTools.buildLayeredImage {
      name = imageName;
      tag = "latest";
      contents = with pkgs; [nodejs_25 bash];
      config = {
        Cmd = ["${pkgs.nodejs_25}/bin/node" "${npmApp}/server/index.mjs"];
        Env = [
          "PORT=${toString port}"
          "NODE_ENV=production"
        ];
        ExposedPorts = {
          "${toString port}/tcp" = {};
        };
      };
    };
}
