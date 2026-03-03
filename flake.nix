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
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        (inputs.import-tree ./nixModules)
      ];
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
    };
}
