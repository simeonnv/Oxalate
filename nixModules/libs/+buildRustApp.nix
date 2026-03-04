{
  inputs,
  lib,
  self,
  ...
}: {
  flake.lib.buildRustApp = {
    pkgs,
    lib,
    ...
  }: name: let
    naersk-lib = pkgs.callPackage inputs.naersk {};
    deps = self.lib.getDeps pkgs lib;
  in
    naersk-lib.buildPackage (
      deps
      // {
        pname = name;
        version = "0.1.0";
        src = ./../..;
        SQLX_OFFLINE = "true";
        SQLX_OFFLINE_DIR = ./../.. + "/.sqlx";
        LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";

        CPATH = "${lib.makeSearchPathOutput "dev" "include" [pkgs.stdenv.cc.cc pkgs.zlib]}";

        preBuild = ''
          export SET_MAKE_JOBS=$NIX_BUILD_CORES
          export GOROOT="${pkgs.go}/share/go"
          export RUST_BACKTRACE=1

          export BINDGEN_EXTRA_CLANG_ARGS="$(< ${pkgs.stdenv.cc}/nix-support/libc-crt1-cflags) \
            $(< ${pkgs.stdenv.cc}/nix-support/libc-cflags) \
            $(< ${pkgs.stdenv.cc}/nix-support/cc-cflags) \
            $(< ${pkgs.stdenv.cc}/nix-support/libc-ldflags) \
            -idirafter ${pkgs.stdenv.cc.cc}/include \
            -idirafter ${pkgs.libclang.lib}/lib/clang/${lib.getVersion pkgs.clang}/include"
        '';

        meta = {
          mainProgram = name;
        };

        cargoBuildOptions = x: x ++ ["-p" name];
        cargoTestOptions = x: x ++ ["-p" name];
      }
    );
}
