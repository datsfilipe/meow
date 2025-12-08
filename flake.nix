{
  description = "A high-performance cat/bat alternative using your existing Neovim configuration";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
        manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = manifest.name;
          version = manifest.version;

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          buildInputs = [];
          nativeBuildInputs = [pkgs.makeWrapper];

          postInstall = ''
            wrapProgram $out/bin/meow \
              --prefix PATH : ${pkgs.lib.makeBinPath [pkgs.less]}
          '';

          meta = with pkgs.lib; {
            description = manifest.description;
            homepage = "https://github.com/datsfilipe/meow";
            license = licenses.mit;
            mainProgram = "meow";
          };
        };

        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rustfmt
            clippy
            hyperfine
          ];
        };
      }
    );
}
