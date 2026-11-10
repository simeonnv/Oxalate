# Copyright 2026 Simeon Vangelov
# SPDX-License-Identifier: GPL-3.0-only

{lib, ...}: {
  options.flake.lib = lib.mkOption {
    type = lib.types.attrsOf lib.types.unspecified;
    default = {};
  };
}
