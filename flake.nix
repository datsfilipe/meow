{
  description = "Meow!";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    system = "x86_64-linux";
  in {
    packages.x86_64-linux.meow = nixpkgs.legacyPackages.${system}.rustPlatform.buildRustPackage {
      version = "1.0.1";
      name = "meow";

      src = ./.;
      cargoLock.lockFile = ./Cargo.lock;
    };

    packages.x86_64-linux.default = self.packages.x86_64-linux.meow;
  };
}
