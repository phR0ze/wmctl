# wmctl nix build configuration
#
# To use this build script:
# 1. Save it as wmctl/default.nix in your configuration files
# 2. Reference it in your systemPackages list
#    environment.systemPackages = [
#      (pkgs.callPackage ./wmctl { })
#    ];
#---------------------------------------------------------------------------------------------------
{ lib, pkgs, ... }: with lib.types;

pkgs.rustPlatform.buildRustPackage rec {
  pname = "wmctl";
  version = "0.0.51";
  src = pkgs.fetchFromGitHub {
    owner = "phR0ze";
    repo = pname;
    rev = "refs/tags/v${version}";
    hash = "sha256-C5MMuoe9FIjXKQvo4vaULxKOFL/yqeZokfa32KjSLLQ=";
  };

  cargoHash = "sha256-wGEi6DASYDEEmPiFTeBarl9EaLQlFoue6USxkHsv2xQ=";
}
