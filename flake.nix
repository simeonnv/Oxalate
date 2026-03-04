{
  description = "Oxalate dev flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    import-tree.url = "github:vic/import-tree";

    naersk.url = "github:nix-community/naersk";

    kubenix.url = "github:hall/kubenix";

    servo.url = "github:simeonnv/Servo";
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        (inputs.import-tree ./nixModules)
        ({lib, ...}: {
          options.flake.kubenixModules = lib.mkOption {
            type = lib.types.lazyAttrsOf lib.types.deferredModule;
            default = {};
          };
        })
      ];
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
    };
}
