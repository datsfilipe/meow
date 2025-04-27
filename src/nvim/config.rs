use std::{
    env, fs,
    io::{self},
    path::PathBuf,
    process::Command,
    sync::OnceLock,
};

use crate::util;

pub const DEFAULT_CONFIG: &str = r#"-- Lua default init for meow
vim.opt.compatible = false

vim.cmd([[filetype off]])
vim.cmd([[filetype plugin indent on]])
vim.cmd([[syntax on]])

local base = vim.fn.expand('~/.local/share/meow')
for _, d in ipairs(vim.fn.glob(base .. '/*', true, true)) do
  vim.opt.runtimepath:append(d)
end
"#;

static BASE_DIR_CACHE: OnceLock<Option<PathBuf>> = OnceLock::new();

pub struct Config {
    pub path: PathBuf,
}

impl Config {
    pub fn new(path: Option<PathBuf>) -> Self {
        let path = match path {
            Some(p) => p,
            None => {
                let cfg_home = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
                    let home = env::var("HOME").expect("HOME not set");
                    format!("{}/.config", home)
                });
                PathBuf::from(cfg_home).join("meow/init.lua")
            }
        };

        let config = Config { path };
        if !config.path.exists() {
            config.init_default().unwrap();
        }

        config
    }

    fn base_dir(&self) -> io::Result<PathBuf> {
        if let Some(dir) = BASE_DIR_CACHE.get() {
            return dir.clone().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "missing data directory, please set XDG_DATA_HOME, nya!",
                )
            });
        }

        let data_dir = match env::var("XDG_DATA_HOME")
            .or_else(|_| env::var("HOME").map(|h| format!("{}/.local/share", h)))
        {
            Ok(dir) => PathBuf::from(dir).join("meow"),
            Err(_) => {
                let err = io::Error::new(
                    io::ErrorKind::NotFound,
                    "missing data directory, please set XDG_DATA_HOME, nya!",
                );
                BASE_DIR_CACHE.get_or_init(|| None);
                return Err(err);
            }
        };

        BASE_DIR_CACHE.get_or_init(|| Some(data_dir.clone()));
        Ok(data_dir)
    }

    fn plugin_dir(&self) -> io::Result<PathBuf> {
        Ok(self.path.parent().unwrap().join("plugin"))
    }

    pub fn init_default(&self) -> io::Result<()> {
        let cfg_dir = self.path.parent().unwrap();
        fs::create_dir_all(cfg_dir)?;
        fs::create_dir_all(self.plugin_dir()?)?;
        fs::create_dir_all(self.base_dir()?)?;
        fs::write(&self.path, DEFAULT_CONFIG)
    }

    pub fn add_colorscheme(&self, repo_spec: &str) -> io::Result<()> {
        if !util::validate::is_git_installed() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "git not found, nyah!",
            ));
        }

        let parts: Vec<&str> = repo_spec.split('/').collect();
        if parts.len() < 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid repository format. Expected USER/REPO, nya!",
            ));
        }

        let user = parts[0];
        let repo = parts[1];
        let mut plugin_url = String::with_capacity(user.len() + repo.len() + 25);
        plugin_url.push_str("https://github.com/");
        plugin_url.push_str(user);
        plugin_url.push('/');
        plugin_url.push_str(repo);
        plugin_url.push_str(".git");

        let name = repo;
        let install = self.base_dir()?.join(name);
        if install.exists() {
            return Ok(());
        }

        fs::create_dir_all(install.parent().unwrap())?;
        let mut cmd = Command::new("git");
        cmd.arg("clone")
            .arg("--depth=1")
            .arg(&plugin_url)
            .arg(install.to_str().unwrap());

        if parts.len() >= 4 && parts[2] == "tree" {
            cmd.arg("--branch").arg(parts[3]);
        }

        cmd.status()?;

        if !install.join("colors").is_dir() {
            fs::remove_dir_all(&install)?;
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "not a colorscheme plugin, nya!",
            ));
        }

        Ok(())
    }

    pub fn remove_colorscheme(&self, repo_spec: &str) -> io::Result<()> {
        let parts: Vec<&str> = repo_spec.split('/').collect();
        if parts.len() < 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid repository format. Expected USER/REPO, nya!",
            ));
        }

        let name = parts[1];
        let target = self.base_dir()?.join(name);

        if target.exists() {
            fs::remove_dir_all(&target)?;
        }

        let colorscheme_file = self.plugin_dir()?.join("colorscheme.lua");
        if colorscheme_file.exists() {
            fs::remove_file(&colorscheme_file)?;
        }

        Ok(())
    }

    pub fn set_colorscheme(&self, repo_spec: &str) -> io::Result<()> {
        let parts: Vec<&str> = repo_spec.split('/').collect();
        if parts.len() < 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid repository format. Expected USER/REPO, nya!",
            ));
        }

        let repo = parts[1];
        let scheme = repo.trim_end_matches(".nvim");
        let plugin_dir = self.plugin_dir()?;
        fs::create_dir_all(&plugin_dir)?;

        let colorscheme_file = plugin_dir.join("colorscheme.lua");
        let mut content = String::with_capacity(40 + scheme.len());
        content.push_str("vim.cmd([[colorscheme ");
        content.push_str(scheme);
        content.push_str("]])");

        fs::write(colorscheme_file, content)?;

        Ok(())
    }
}
