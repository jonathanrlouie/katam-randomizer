{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    crate2nix = {
      url = "github:kolloch/crate2nix";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, flake-utils, crate2nix }:
    flake-utils.lib.eachDefaultSystem (system:
      let pkgs = nixpkgs.legacyPackages.${system};
        crateName = "katam-randomizer";
        inherit (import "${crate2nix}/tools.nix" { inherit pkgs; })
          generatedCargoNix;
        project = import (generatedCargoNix {
          name = crateName;
          src = ./.;
        }) { inherit pkgs; };
      in {
        packages.playground = project.rootCrate.build;
        devShell = pkgs.mkShell { buildInputs = [ project.rootCrate.build ]; };
      }
    );
}
