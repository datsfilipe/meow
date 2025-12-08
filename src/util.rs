use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

pub fn is_binary_or_device(path: &Path) -> io::Result<bool> {
    let metadata = fs::metadata(path)?;

    if metadata.is_dir() {
        return Err(io::Error::new(io::ErrorKind::Other, "Is a directory"));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;
        if metadata.file_type().is_char_device() || metadata.file_type().is_block_device() {
            return Ok(true);
        }
    }

    let mut file = File::open(path)?;
    let mut buffer = [0; 1024];
    let n = file.read(&mut buffer)?;

    if buffer[..n].contains(&0) {
        return Ok(true);
    }

    Ok(false)
}

pub fn file_exceeds_terminal_height(path: &Path) -> io::Result<bool> {
    let (_, h) = term_size::dimensions().unwrap_or((80, 24));

    let file = File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let mut count = 0;

    use std::io::BufRead;
    for _ in reader.lines() {
        count += 1;
        if count > h {
            return Ok(true);
        }
    }

    Ok(false)
}

pub fn write_temp_lua_script(content: &str) -> io::Result<PathBuf> {
    let mut temp_path = std::env::temp_dir();
    let pid = std::process::id();
    temp_path.push(format!("meow_{}.lua", pid));

    fs::write(&temp_path, content)?;
    Ok(temp_path)
}
