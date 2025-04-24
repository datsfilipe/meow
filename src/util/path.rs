pub fn get_nvim_bin_path() -> String {
    let output = std::process::Command::new("which")
        .arg("nvim")
        .output()
        .expect("failed to execute process");
    let path = String::from_utf8(output.stdout).unwrap();
    path.trim().to_string()
}
