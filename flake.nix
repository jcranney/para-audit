{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, utils }: 
    utils.lib.eachDefaultSystem (system: 
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          name = "para-audit";
          cargoLock.lockFile = ./Cargo.lock;
          src = ./.;
          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = with pkgs; [ openssl ];
          meta = {
            description = "A tool for auditing/organising/interacting with my para system.";
            homepage = "https://github.com/jcranney/para-audit";
            mainProgram = "para";
          };
        };
      }
    );
}