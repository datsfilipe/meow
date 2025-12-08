mod lua;
mod util;

use clap::Parser;
use std::fs::{self, File};
use std::io::{self, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;

#[cfg(unix)]
use std::os::unix::fs::FileTypeExt;

const MAX_HIGHLIGHT_SIZE: u64 = 1024 * 100;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(required = true)]
    files: Vec<PathBuf>,

    #[arg(long, short = 'f')]
    force_color: bool,
}

fn main() {
    let args = Args::parse();

    let lua_script_path = match util::write_temp_lua_script(lua::LUA_GENERATOR) {
        Ok(p) => Arc::new(p),
        Err(e) => {
            eprintln!("meow: failed to write temp script: {}", e);
            std::process::exit(1);
        }
    };

    let use_pager_if_needed = std::io::stdout().is_terminal() && args.files.len() == 1;
    let force_color = args.force_color;
    let files = args.files;
    let mut handles = Vec::with_capacity(files.len());

    for (_index, file_path) in files.into_iter().enumerate() {
        let script_ref = Arc::clone(&lua_script_path);
        let thread_path = file_path.clone();

        let handle = thread::spawn(move || -> Result<Vec<u8>, String> {
            match process_file_capture(&thread_path, &script_ref, use_pager_if_needed, force_color)
            {
                Ok(data) => Ok(data),
                Err(e) => Err(format!("meow: {}: {}", thread_path.display(), e)),
            }
        });

        handles.push((file_path, handle));
    }

    let total_files = handles.len();
    for (i, (path, handle)) in handles.into_iter().enumerate() {
        if total_files > 1 {
            if i > 0 {
                println!("\n");
            }
            println!("\x1b[1;34m:: {} ::\x1b[0m", path.display());
        }

        match handle.join() {
            Ok(result) => match result {
                Ok(bytes) => {
                    if !bytes.is_empty() {
                        let mut stdout = io::stdout().lock();
                        let _ = stdout.write_all(&bytes);
                    }
                }
                Err(e) => eprintln!("{}", e),
            },
            Err(_) => eprintln!("meow: thread panicked for {}", path.display()),
        }
    }

    if let Ok(path) = Arc::try_unwrap(lua_script_path) {
        let _ = fs::remove_file(path);
    }
}

fn process_file_capture(
    path: &Path,
    script_path: &Path,
    allow_pager: bool,
    force_color: bool,
) -> io::Result<Vec<u8>> {
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No such file or directory",
        ));
    }

    let metadata = fs::metadata(path)?;

    #[cfg(unix)]
    let is_device = metadata.file_type().is_char_device() || metadata.file_type().is_block_device();

    #[cfg(not(unix))]
    let is_device = false;

    if is_device {
        let mut file = File::open(path)?;
        let mut stdout = io::stdout().lock();
        io::copy(&mut file, &mut stdout)?;
        return Ok(Vec::new());
    }

    let size = metadata.len();
    let is_too_big = size > MAX_HIGHLIGHT_SIZE;
    let is_binary = util::is_binary_or_device(path)?;

    if is_binary || (is_too_big && !force_color) {
        if is_too_big && allow_pager && util::file_exceeds_terminal_height(path).unwrap_or(false) {
            let mut less = Command::new("less").arg(path).spawn()?;
            less.wait()?;
            return Ok(Vec::new());
        }

        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        return Ok(buffer);
    }

    let canonical_path = path.canonicalize()?;
    let path_str = canonical_path.to_str().ok_or(io::Error::new(
        io::ErrorKind::InvalidData,
        "Path is not valid UTF-8",
    ))?;

    let lua_cmd = format!("luafile {}", script_path.display());

    let mut cmd = Command::new("nvim");
    cmd.arg("--headless")
        .args(["--noplugin", "-c", "set shortmess+=I"])
        .args(["-n", "-i", "NONE"])
        .env("MEOW_FILE", path_str)
        .args(["-c", &lua_cmd]);

    if allow_pager && util::file_exceeds_terminal_height(path).unwrap_or(false) {
        cmd.stdout(Stdio::piped());
        let mut child = cmd.spawn()?;
        let stdout = child.stdout.take().unwrap();

        let mut less = Command::new("less").arg("-R").stdin(stdout).spawn()?;
        less.wait()?;
        child.wait()?;
        return Ok(Vec::new());
    } else {
        let output = cmd.output()?;
        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Neovim exited with error",
            ));
        }
        return Ok(output.stdout);
    }
}
