use crate::util;

use nvim_rs::{Neovim, Value, compat::tokio::Compat, create::tokio as create, error::LoopError};
use std::path::Path;
use tokio::process::ChildStdin;

#[derive(Debug, Default)]
struct HighlightInfo {
    fg: Option<u32>,
    bg: Option<u32>,
    bold: bool,
    italic: bool,
    underline: bool,
}

const EXTRACT_HL_LUA: &str = include_str!("./conf/extract_hl.lua");

pub struct Nvim {
    instance: Neovim<Compat<ChildStdin>>,
    config_path: String,
    _io: tokio::task::JoinHandle<Result<(), Box<LoopError>>>,
    _child: tokio::process::Child,
}

impl Drop for Nvim {
    fn drop(&mut self) {
        self._io.abort();
        let _ = self._child.start_kill();
    }
}

impl Nvim {
    pub async fn new(config: &str) -> Self {
        let config_dir = Path::new(config)
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .to_str()
            .unwrap_or("");

        let (n, io, c) = create::new_child_cmd(
            tokio::process::Command::new(util::path::get_nvim_bin_path())
                .args(&["--embed", "--noplugin"]),
            nvim_rs::rpc::handler::Dummy::new(),
        )
        .await
        .unwrap();
        Self {
            config_path: config_dir.to_string(),
            instance: n,
            _io: io,
            _child: c,
        }
    }

    pub async fn print_file_with_highlighting(
        &self,
        file_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let abs = Path::new(file_path).canonicalize()?;
        let path = abs.to_str().ok_or("invalid UTF-8 in path")?;
        let mut del = None;

        let res = async {
            let buf = self.instance.create_buf(false, true).await.unwrap();
            let num = buf.get_number().await?;
            del = Some(num);
            self.instance.set_current_buf(&buf).await?;

            let esc = path.replace(' ', r"\ ");
            self.instance.command(&format!("edit {}", esc)).await?;

            self.instance.command("syntax enable").await?;
            self.instance.command("set termguicolors").await?;

            self.instance
                .command(format!("source {}/init.lua", self.config_path).as_str())
                .await?;
            self.instance
                .command(format!("source {}/plugin/colorscheme.lua", self.config_path).as_str())
                .await?;

            let highlighted_lines = self.instance.execute_lua(EXTRACT_HL_LUA, vec![]).await?;
            if let Value::Array(lines) = highlighted_lines {
                for line_data in lines {
                    if let Value::Map(line_map) = line_data {
                        let segments = line_map
                            .iter()
                            .find(|(k, _)| k.as_str() == Some("segments"))
                            .and_then(|(_, v)| v.as_array().cloned())
                            .unwrap_or_default();

                        let mut output_line = String::new();

                        for segment in segments {
                            if let Value::Map(segment_map) = segment {
                                let text = segment_map
                                    .iter()
                                    .find(|(k, _)| k.as_str() == Some("text"))
                                    .and_then(|(_, v)| v.as_str())
                                    .unwrap_or("");

                                let hl: Vec<_> = segment_map
                                    .iter()
                                    .find(|(k, _)| k.as_str() == Some("hl"))
                                    .and_then(|(_, v)| v.as_map().cloned())
                                    .unwrap_or_default();

                                let fg = hl
                                    .iter()
                                    .find(|(k, _)| k.as_str() == Some("fg"))
                                    .and_then(|(_, v)| v.as_u64())
                                    .map(|v| v as u32);

                                let bg = hl
                                    .iter()
                                    .find(|(k, _)| k.as_str() == Some("bg"))
                                    .and_then(|(_, v)| v.as_u64())
                                    .map(|v| v as u32);

                                let bold = hl
                                    .iter()
                                    .find(|(k, _)| k.as_str() == Some("bold"))
                                    .and_then(|(_, v)| v.as_bool())
                                    .unwrap_or(false);

                                let italic = hl
                                    .iter()
                                    .find(|(k, _)| k.as_str() == Some("italic"))
                                    .and_then(|(_, v)| v.as_bool())
                                    .unwrap_or(false);

                                let underline = hl
                                    .iter()
                                    .find(|(k, _)| k.as_str() == Some("underline"))
                                    .and_then(|(_, v)| v.as_bool())
                                    .unwrap_or(false);

                                let hl_info = HighlightInfo {
                                    fg,
                                    bg,
                                    bold,
                                    italic,
                                    underline,
                                };

                                let ansi_codes = self.ansi(&hl_info);
                                output_line.push_str(&ansi_codes);
                                output_line.push_str(text);
                                output_line.push_str("\x1b[0m");
                            }
                        }

                        println!("{}", output_line);
                    }
                }
            }

            Ok::<(), Box<dyn std::error::Error>>(())
        }
        .await;

        if let Some(n) = del {
            let _ = self
                .instance
                .command(&format!("silent! bdelete! {}", n))
                .await;
        }
        res.map_err(|e| {
            eprintln!("{}, {}", e, "nyah!");
            std::process::exit(1)
        })
    }

    fn ansi(&self, hl: &HighlightInfo) -> String {
        let mut codes = Vec::new();
        codes.push("0".to_string());

        if hl.bold {
            codes.push("1".to_string());
        }
        if hl.italic {
            codes.push("3".to_string());
        }
        if hl.underline {
            codes.push("4".to_string());
        }

        if let Some(color) = hl.fg {
            let r = (color >> 16) & 0xFF;
            let g = (color >> 8) & 0xFF;
            let b = color & 0xFF;
            codes.push(format!("38;2;{};{};{}", r, g, b));
        }

        if let Some(color) = hl.bg {
            let r = (color >> 16) & 0xFF;
            let g = (color >> 8) & 0xFF;
            let b = color & 0xFF;
            codes.push(format!("48;2;{};{};{}", r, g, b));
        }

        format!("\x1b[{}m", codes.join(";"))
    }
}
