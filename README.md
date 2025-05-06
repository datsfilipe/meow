# Meow

## Description

Meow uses neovim text editor to print highlighted text in the terminal. Yeah, like `cat`, `bat`, etc. But with neovim, which allow it to be more configurable, since it uses lua. The tool config is managed in another location, out

## Preview

https://github.com/user-attachments/assets/b0e3a2c6-b69d-4e66-9d00-9b8727deaf45

## Installation

If the following is not applicable for you, you can just grab a binary from the [releases](https://github.com/datsfilipe/meow/releases) page.

### Arch Linux

1. Install it from the [AUR](https://aur.archlinux.org/packages/meow-nvim) with your favorite aur helper.
2. Example with paru:

```bash
paru -Syu meow-nvim
```

**Note**: Special thanks to [@fk29g](https://github.com/fk29g) for creating and maintaining the [meow-nvim](https://aur.archlinux.org/packages/meow-nvim) AUR package.

### NixOS

1. Add it to your flake inputs:

```nix
nixpkgs-unstable.url = "github:nixos/nixpkgs/nixos-unstable";
meow = {
  inputs.nixpkgs.follows = "nixpkgs-unstable";
  url = "github:datsfilipe/meow/main";
};
```

And then add it the following to you configuration:

```nix
{
  pkgs,
  meow,
  ...
}: {
  home.packages = [
    meow.packages.${pkgs.system}.default
  ];
}
```

2. Or you can install it in a less declarative way with a single command:

```bash
nix profile install github:datsfilipe/meow/main
```

**Note**: you can also just run it with:

```bash
nix run github:datsfilipe/meow/main
```

## Usage  

```bash
usage:
  bin [FILE]
  bin --config PATH [FILE]
  bin --add-colorscheme USER/REPO(/TREE/BRANCH)
  bin --set-colorscheme USER/REPO
  bin --remove-colorscheme USER/REPO

note: colorscheme commands cannot be combined with each other or with file arguments, nya!
```

## To Do

- [X] Handle outputs bigger than terminal screen with less scrolling

## Contributing  

Guidelines for contributing to the project:  

1. Fork the repository.  
2. Make your changes.  
3. Submit a pull request.
4. Nothing fancy, contributions are welcome.

## Aknowledgements

- [nvim-cat](https://github.com/lincheney/nvim-cat) - The inspiration for this project

## License  

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
