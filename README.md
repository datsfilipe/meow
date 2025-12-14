<div align="center">

# Meow

Meow is a terminal printing tool that uses a headless Neovim instance to handle syntax highlighting. It provides a simple way to get editor-quality colors in your terminal using your local colorscheme without managing separate bat or pygments configs.

</div>

## Preview

https://github.com/user-attachments/assets/298e6135-b01e-454f-bbed-0f363dce52fa

## Features

- **Neovim Highlighting**: Uses Neovim's syntax engine and your active MEOW_THEME or system colorscheme.
- **Parallel Processing**: Multithreaded rendering for high performance.
- **Built-in Pager**: Interactive TUI pager for files that exceed terminal height.
- **Raw Streaming**: cat-equivalent speed for binary files and devices (e.g., /dev/input/mice).
- **Fast Mode**: Automatically skips highlighting for large files to eliminate latency.

## Installation

### NixOS / Home Manager (Flake)

Add `meow` to your `flake.nix` inputs:

```nix
inputs.meow.url = "github:datsfilipe/meow";
```

Then add it to your system packages:

```nix
environment.systemPackages = [
  inputs.meow.packages.${pkgs.system}.default
];
```

### Cargo

```bash
cargo install rmeow
```

## Usage

```bash
# standard highlight (uses Neovim)
meow src/main.rs

# multiple files (parallel processing)
meow src/*.rs

# force highlighting on large files (bypass fast path)
meow -f assets/huge_file.lua

# raw device streaming (zero overhead)
meow /dev/input/mice
```

## Benchmarks

**1. Syntax Highlighting (Large File)**

| Command | Time | Result |
| :--- | :--- | :--- |
| `meow --force-color` | **~199.2 ms** | **1.0x (Winner)** |
| `bat --color=always` | ~1.476 s | 7.41x slower |

**2. Standard Printing (Fast Path)**

| Command | Time | Result |
| :--- | :--- | :--- |
| `bat` | **~12.7 ms** | **1.0x (Winner)** |
| `meow` | ~52.9 ms | 4.16x slower |

**3. Raw Throughput** (*Streaming `/dev/input/mice` for 5 seconds*)

| Command | Data Moved | Efficiency |
| :--- | :--- | :--- |
| `cat` | 101,232 bytes | 100% |
| `meow` | 103,323 bytes | ~100% |

## Acknowledgements

- [nvim-cat](https://github.com/lincheney/nvim-cat) - inspiration for the initial idea of this project.

## License

This project is licensed under the [MIT License](./LICENSE).
