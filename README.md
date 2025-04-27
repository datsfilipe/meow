# Meow

## Description

Meow uses neovim text editor to print highlighted text in the terminal. Yeah, like `cat`, `bat`, etc. But with neovim, which allow it to be more configurable, since it uses lua. The tool config is managed in another location, out

## Preview

https://github.com/user-attachments/assets/b0e3a2c6-b69d-4e66-9d00-9b8727deaf45

## Installation  

Grab a binary from the [releases](https://github.com/datsfilipe/meow/releases) page. In future will try to add to some package managers.

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
