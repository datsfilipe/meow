use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn write_temp_lua_script(content: &str) -> io::Result<PathBuf> {
    let mut path = std::env::temp_dir();
    path.push(format!("meow_script_{}.lua", std::process::id()));
    std::fs::write(&path, content)?;
    Ok(path)
}

pub fn is_binary_or_device(path: &Path) -> io::Result<bool> {
    let mut file = File::open(path)?;
    let mut buffer = [0; 8192];
    let n = file.read(&mut buffer)?;

    for &b in &buffer[..n] {
        if b == 0 {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn file_exceeds_terminal_height(path: &Path) -> io::Result<bool> {
    let output = Command::new("tput").arg("lines").output();

    let height = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .trim()
            .parse::<usize>()
            .unwrap_or(24),
        _ => 24,
    };

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut lines = 0;
    for _ in reader.lines() {
        lines += 1;
        if lines > height {
            return Ok(true);
        }
    }

    Ok(false)
}
