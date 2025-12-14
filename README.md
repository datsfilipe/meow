<div align="center">

# Meow

## Description

**Meow** is a terminal printing tool that renders text using your **existing Neovim configuration**.

While great tools like `bat` exist, they use their own syntax engines. **Meow** is for users who want their terminal output to look exactly like their editor—colorscheme, custom highlights, and all—without managing a second set of configs. It balances this Neovim-fidelity with high performance via a custom Lua generator and Rust multithreading.

</div>

## Preview

https://github.com/user-attachments/assets/298e6135-b01e-454f-bbed-0f363dce52fa

## Features

- **Neovim Engine:** Uses your local `~/.config/nvim`. If you've spent hours rice-ing your editor, Meow lets you see that effort everywhere.
- **Parallel Rendering:** Uses all CPU threads to process files.
- **Built-in Pager:** Seamlessly transitions to a TUI pager when files exceed the terminal height.
- **Device Support:** Handles `/dev/input/mice` and other character devices with `cat`-like streaming.
- **Fast Path:** Transparently skips highlighting for massive files unless you explicitly ask for it, keeping the tool snappy for general use.

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

Transparency is key: `bat` is a highly optimized tool. When no highlighting is required, `bat` is significantly faster. However, when it comes to the heavy lifting of full syntax highlighting on large files, Meow's parallelized Neovim engine takes the lead.

**1. Heavy Highlighting (Large File)**
*Rendering a 1.2MB Lua file with full color*

| Command | Time | Result |
| :--- | :--- | :--- |
| `meow --force-color` | **~199.2 ms** | **1.0x (Winner)** |
| `bat --color=always` | ~1.476 s | 7.41x slower |

**2. Standard Printing (Fast Path)**
*Printing a 1.2MB Lua file without highlighting*

| Command | Time | Result |
| :--- | :--- | :--- |
| `bat` | **~12.7 ms** | **1.0x (Winner)** |
| `meow` | ~52.9 ms | 4.16x slower |

**3. Raw Throughput**
*Streaming `/dev/input/mice` for 5 seconds*

| Command | Data Moved | Efficiency |
| :--- | :--- | :--- |
| `cat` | 101,232 bytes | 100% |
| `meow` | 103,323 bytes | ~100% |

## Acknowledgements

- [nvim-cat](https://github.com/lincheney/nvim-cat) - The inspiration for this project.

## License

This project is licensed under the MIT License.
