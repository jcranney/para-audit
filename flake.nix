{
  description = "set up rust toolchain for developing and packaging";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, naersk }: let 
    pkgs = nixpkgs.legacyPackages."x86_64-linux";
    naerskLib = pkgs.callPackage naersk {};
    in {
      devShells."x86_64-linux".default = pkgs.mkShell {
        buildInputs = with pkgs; [
          cargo rustc rustfmt clippy rust-analyzer openssl
        ];

        nativeBuildInputs = [ pkgs.pkg-config ];
        env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
      };
      packages."x86_64-linux".default = pkgs.callPackage ./default.nix {};

      # packages."x86_64-linux".default = naerskLib.buildPackage {
      #   src = ./.;
      #   buildInputs = [ pkgs.glib pkgs.openssl ];
      #   nativeBuildInputs = [ pkgs.pkg-config ];
      # };
    };
}
