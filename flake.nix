{
  description = "Qorx language and runtime for local context resolution";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "qorx";
            version = "1.0.2";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            doCheck = true;
            meta = with pkgs.lib; {
              description = "Qorx language and runtime for local context resolution";
              homepage = "https://github.com/bbrainfuckk/qorx";
              license = licenses.agpl3Only;
              mainProgram = "qorx";
            };
          };
        });

      apps = forAllSystems (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/qorx";
        };
      });

      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.mkShell {
            packages = [ pkgs.cargo pkgs.rustc pkgs.rustfmt pkgs.clippy ];
          };
        });
    };
}
