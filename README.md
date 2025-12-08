# Meow

<div align="center">

## Description

**Meow** is a terminal printing tool (like `cat` or `bat`) that renders text using your **existing Neovim configuration**.

It spawns a headless Neovim instance to render files exactly as you see them in your editor—colorscheme, syntax highlighting, and all—while maintaining high performance via a custom Lua rendering engine and Rust multithreading.

</div>

## Preview

<video src="./assets/preview.mp4" controls="controls" style="max-width: 100%;">
</video>

## Features

- **Neovim Engine:** Uses your local `~/.config/nvim` (no separate config required).
- **Parallel Processing:** Renders multiple files simultaneously using all CPU threads.
- **Smart Paging:** Automatically pipes to `less` only if the file exceeds your terminal height.
- **Zero Overhead:** Detects binary files and devices (like `/dev/input/mice`) and streams them with raw `cat` speed.
- **Performance:** Includes a "Fast Mode" for large files (>100KB) to skip rendering latency, unless forced.

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

Or install declaratively in your `home-manager` config.

### Manual

You can run it directly without installing:

```bash
nix run github:datsfilipe/meow
```

## Usage

```bash
# Print a single file (highlights automatically)
meow src/main.rs

# Print multiple files (renders in parallel)
meow src/*.rs

# Force syntax highlighting on large files (>100KB)
meow -f assets/huge_file.lua

# Stream devices (zero overhead)
meow /dev/input/mice
```

## Benchmarks

I remember...

<div align="center">

!["Can it do 'sudo cat /dev/input/mice'?" comment on reddit post.](./assets/r_comment.jpg)

</div>

Now it can!

**1. Throughput (Raw Streaming)**
*Meow vs Cat streaming `/dev/input/mice` (Infinite Binary Stream)*

| Command | Data Moved (5s) | Overhead |
| :--- | :--- | :--- |
| `cat` | ~104 KB | 0% |
| `meow` | ~99 KB | ~0% |

**2. Syntax Highlighting (Large File)**
*Meow vs Bat rendering a 2.5MB Lua file*

| Command | Time | Relative Speed |
| :--- | :--- | :--- |
| `meow --force-color` | **699.9 ms** | **1.0x** |
| `bat --paging=never --style=plain --color=always` | 2.254 s | 3.2x slower |

*Note: `cat` (no highlight) takes ~1ms. That is not beatable. But we did beat bat :)*

## Acknowledgements

- [nvim-cat](https://github.com/lincheney/nvim-cat) - The inspiration for this project

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
