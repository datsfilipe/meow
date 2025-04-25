use std::{
    env, fs,
    io::{self},
    path::PathBuf,
    process::Command,
};

pub const DEFAULT_CONFIG: &str = r#"-- Lua default init for nv-meow
vim.opt.compatible = false

vim.cmd([[filetype off]])
vim.cmd([[filetype plugin indent on]])
vim.cmd([[syntax on]])

local base = vim.fn.expand('~/.local/share/nv-meow')
for _, d in ipairs(vim.fn.glob(base .. '/*', true, true)) do
  vim.opt.runtimepath:append(d)
end
"#;

pub struct Config {
    pub path: PathBuf,
}

impl Config {
    pub fn new(path: Option<PathBuf>) -> Self {
        let path = match path {
            Some(p) => p,
            None => {
                let cfg_home = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
                    let home = env::var("HOME").unwrap();
                    format!("{}/.config", home)
                });
                PathBuf::from(cfg_home).join("nv-meow/init.lua")
            }
        };

        let config = Config { path };
        if !config.path.exists() {
            config.init_default().unwrap();
        }

        config
    }

    fn base_dir(&self) -> io::Result<PathBuf> {
        let data = match env::var("XDG_DATA_HOME")
            .or_else(|_| env::var("HOME").map(|h| format!("{}/.local/share", h)))
        {
            Ok(dir) => dir,
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "missing data directory, please set XDG_DATA_HOME! nyah",
                ));
            }
        };

        Ok(PathBuf::from(data).join("nv-meow"))
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
        let parts: Vec<&str> = repo_spec.split('/').collect();
        if parts.len() < 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid repository format. Expected USER/REPO",
            ));
        }

        let user = parts[0];
        let repo = parts[1];
        let plugin_url = format!("https://github.com/{}/{}.git", user, repo);

        let name = repo;
        let install = self.base_dir()?.join(name);

        if install.exists() {
            return Ok(());
        }

        fs::create_dir_all(install.parent().unwrap())?;
        let mut cmd = Command::new("git");
        cmd.args(&["clone", "--depth=1", &plugin_url, install.to_str().unwrap()]);

        if parts.len() >= 4 && parts[2] == "tree" {
            cmd.args(&["--branch", parts[3]]);
        }

        cmd.status()?;

        if !install.join("colors").is_dir() {
            fs::remove_dir_all(&install)?;
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Not a colorscheme plugin",
            ));
        }

        Ok(())
    }

    pub fn remove_colorscheme(&self, repo_spec: &str) -> io::Result<()> {
        let parts: Vec<&str> = repo_spec.split('/').collect();
        if parts.len() < 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid repository format. Expected USER/REPO",
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
                "Invalid repository format. Expected USER/REPO",
            ));
        }

        let repo = parts[1];
        let scheme = repo.trim_end_matches(".nvim").to_string();
        let plugin_dir = self.plugin_dir()?;
        fs::create_dir_all(&plugin_dir)?;

        let colorscheme_file = plugin_dir.join("colorscheme.lua");
        fs::write(
            colorscheme_file,
            format!("vim.cmd([[colorscheme {}]])", scheme),
        )?;

        Ok(())
    }
}
