{
  pkgs,
  lib,
  inputs,
  ...
}: let
  inherit (inputs.self.lib) getDeps;
in {
  perSystem = {
    config,
    pkgs,
    system,
    ...
  }: let
    deps = getDeps pkgs lib;
  in {
    devShells.default = pkgs.mkShell {
      nativeBuildInputs = (deps.nativeBuildInputs or []) ++ (deps.devTools or []);
      buildInputs = deps.buildInputs or [];

      shellHook = ''
        export RUST_SRC_PATH="${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}"
        export SQLX_OFFLINE="true"
        export LIBCLANG_PATH="${pkgs.libclang.lib}/lib"
        export LD_LIBRARY_PATH="${pkgs.libclang.lib}/lib:${pkgs.llvm.lib}/lib''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

        echo "We are so nix brur"
      '';
    };
  };
}
