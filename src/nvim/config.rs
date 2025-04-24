use std::{
    env, fs,
    io::{self, Write},
    path::PathBuf,
    process::Command,
};

pub const DEFAULT_CONFIG: &str = r#"-- Lua default init for nv-cat
vim.opt.compatible = false

vim.cmd([[filetype off]])

vim.cmd([[call pack#init()]])

vim.cmd([[filetype plugin indent on]])
vim.cmd([[syntax on]])
"#;

pub struct Config {
    pub path: PathBuf,
}

impl Config {
    pub fn new(path: Option<impl Into<PathBuf>>) -> Self {
        let path = match path {
            Some(p) => p.into(),
            None => {
                let cfg_home = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
                    let home = env::var("HOME").unwrap();
                    format!("{}/.config", home)
                });
                PathBuf::from(cfg_home).join("nv-cat/init.lua")
            }
        };
        Config { path }
    }

    pub fn init_default(&self) -> io::Result<()> {
        let cfg_dir = self.path.parent().unwrap();
        fs::create_dir_all(cfg_dir)?;
        fs::write(&self.path, DEFAULT_CONFIG)
    }
}
