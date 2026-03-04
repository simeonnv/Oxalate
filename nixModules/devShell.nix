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

      LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";

      CPATH = "${lib.makeSearchPathOutput "dev" "include" [pkgs.stdenv.cc.cc pkgs.zlib]}";

      shellHook = ''
        export RUST_SRC_PATH="${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}"
        export SQLX_OFFLINE="true"
        export LIBCLANG_PATH="${pkgs.libclang.lib}/lib"
        export LD_LIBRARY_PATH="${pkgs.libclang.lib}/lib:${pkgs.llvm.lib}/lib''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
        export GOROOT="${pkgs.go}/share/go"

        export BINDGEN_EXTRA_CLANG_ARGS="$(< ${pkgs.stdenv.cc}/nix-support/libc-crt1-cflags) \
          $(< ${pkgs.stdenv.cc}/nix-support/libc-cflags) \
          $(< ${pkgs.stdenv.cc}/nix-support/cc-cflags) \
          $(< ${pkgs.stdenv.cc}/nix-support/libc-ldflags) \
          -idirafter ${pkgs.stdenv.cc.cc}/include \
          -idirafter ${pkgs.libclang.lib}/lib/clang/${lib.getVersion pkgs.clang}/include"

        echo "We are so nix brur"
      '';
    };
  };
}
