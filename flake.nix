{
  description = "Qorx Community Edition CLI";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = f:
        builtins.listToAttrs (map (system: {
          name = system;
          value = f system;
        }) systems);
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "qorx";
            version = "1.0.4a";
            src = self;
            cargoLock.lockFile = ./Cargo.lock;
            meta = {
              description = "Qorx Community Edition CLI";
              homepage = "https://github.com/bbrainfuckk/qorx";
              license = pkgs.lib.licenses.agpl3Only;
              mainProgram = "qorx";
            };
          };
        });
    };
}
