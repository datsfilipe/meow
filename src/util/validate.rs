pub fn is_nvim_installed() -> bool {
    let output = std::process::Command::new("nvim")
        .arg("--version")
        .output()
        .expect("failed to execute process");

    output.status.success()
}

pub fn is_git_installed() -> bool {
    let output = std::process::Command::new("git")
        .arg("--version")
        .output()
        .expect("failed to execute process");

    output.status.success()
}
