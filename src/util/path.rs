use std::{process::Command, sync::OnceLock};

static NVIM_PATH: OnceLock<String> = OnceLock::new();

pub fn get_nvim_bin_path() -> &'static str {
    NVIM_PATH.get_or_init(|| {
        let output = Command::new("which")
            .arg("nvim")
            .output()
            .expect("failed to execute process");

        String::from_utf8_lossy(&output.stdout).trim().to_string()
    })
}
