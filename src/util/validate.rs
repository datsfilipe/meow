use std::{process::Command, sync::OnceLock};

static NVIM_INSTALLED: OnceLock<bool> = OnceLock::new();
static GIT_INSTALLED: OnceLock<bool> = OnceLock::new();

pub fn is_less_installed() -> bool {
    *GIT_INSTALLED.get_or_init(|| {
        Command::new("less")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    })
}

pub fn is_nvim_installed() -> bool {
    *NVIM_INSTALLED.get_or_init(|| {
        Command::new("nvim")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    })
}

pub fn is_git_installed() -> bool {
    *GIT_INSTALLED.get_or_init(|| {
        Command::new("git")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    })
}
