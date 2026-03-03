{
  inputs,
  lib,
  self,
  ...
}: {
  flake.lib.buildNpmApp = {
    pkgs,
    depsHash ? pkgs.lib.fakeHash,
    ...
  }: name: src:
    pkgs.buildNpmPackage {
      pname = name;
      version = "0.1.0";
      src = src;

      NUXT_TELEMETRY_DISABLED = 1;

      preBuild = ''
        rm -rf .nuxt
      '';
      makeCacheWritable = true;
      npmFlags = ["--legacy-peer-deps"];
      npmInstallFlags = ["--include=optional"];

      npmDepsHash = depsHash;

      installPhase = ''
        runHook preInstall
        mkdir -p $out/bin

        cp -r .output/* $out/
        echo "#!/bin/sh
        exec ${pkgs.nodejs}/bin/node $out/server/index.mjs" > $out/bin/${name}
        chmod +x $out/bin/${name}

        runHook postInstall
      '';

      meta = {
        mainProgram = name;
      };
    };
}
