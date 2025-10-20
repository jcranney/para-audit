{
    rustPlatform, glib, pkg-config, openssl
}:

rustPlatform.buildRustPackage {
    name = "para";
    src = ./.;
    buildInputs = [
        glib openssl
    ];
    nativeBuildInputs = [ pkg-config ];
    cargoHash = "sha256-mmmm4m2c5IxBzMBMIk6zVZHTnHpQZyR8CBXDir3PhNc=";
}