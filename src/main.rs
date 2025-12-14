mod lua;
mod util;

use clap::{Parser, ValueEnum};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute, queue,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, OnceLock, mpsc};
use std::thread;
use std::time::Duration;

#[cfg(unix)]
use std::os::unix::fs::FileTypeExt;

const CHUNK_THRESHOLD_BYTES: u64 = 50 * 1024; // 50KB
const MAX_HIGHLIGHT_SIZE: u64 = 1024 * 1024; // 1MB

#[derive(Debug, Clone, ValueEnum, PartialEq)]
enum PagerMode {
    Auto,
    Never,
    No,
    Always,
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(required = true)]
    files: Vec<PathBuf>,

    #[arg(long, short = 'f')]
    force_color: bool,

    #[arg(long)]
    theme: Option<String>,

    #[arg(long, short = 'p', default_value = "auto", value_enum)]
    pager: PagerMode,
}

#[derive(Debug, Clone)]
struct NvimInfo {
    theme: String,
    rtp: String,
}

static LUA_SCRIPT_PATH: OnceLock<Arc<PathBuf>> = OnceLock::new();
static NVIM_INFO: OnceLock<NvimInfo> = OnceLock::new();

fn get_lua_script() -> Arc<PathBuf> {
    LUA_SCRIPT_PATH
        .get_or_init(|| match util::write_temp_lua_script(lua::LUA_GENERATOR) {
            Ok(p) => Arc::new(p),
            Err(e) => {
                eprintln!("meow: failed to write temp script: {}", e);
                std::process::exit(1);
            }
        })
        .clone()
}

fn get_nvim_info(theme_arg: Option<String>) -> NvimInfo {
    NVIM_INFO
        .get_or_init(|| {
            if let Some(t) = theme_arg {
                NvimInfo {
                    theme: t,
                    rtp: "".to_string(),
                }
            } else if let Ok(t) = std::env::var("MEOW_THEME") {
                NvimInfo {
                    theme: t,
                    rtp: "".to_string(),
                }
            } else {
                get_neovim_info_safe().unwrap_or(NvimInfo {
                    theme: "habamax".to_string(),
                    rtp: "".to_string(),
                })
            }
        })
        .clone()
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let is_tty = io::stdout().is_terminal();
    let multiple_files = args.files.len() > 1;

    for (i, file_path) in args.files.iter().enumerate() {
        let metadata = match fs::metadata(file_path) {
            Ok(m) => m,
            Err(e) => {
                eprintln!(
                    "meow: could not read metadata for {}: {}",
                    file_path.display(),
                    e
                );
                continue;
            }
        };

        #[cfg(unix)]
        let is_device =
            metadata.file_type().is_char_device() || metadata.file_type().is_block_device();
        #[cfg(not(unix))]
        let is_device = false;

        if is_device {
            if multiple_files {
                let mut out = io::stdout().lock();
                if i > 0 {
                    let _ = out.write_all(b"\n\n");
                }
                let _ = out.write_all(
                    format!("\x1b[1;34m:: {} ::\x1b[0m\n", file_path.display()).as_bytes(),
                );
            }
            let mut f = File::open(file_path)?;
            let mut out = io::stdout().lock();
            io::copy(&mut f, &mut out)?;
            continue;
        }

        let size = metadata.len();
        let skip_highlight = size > MAX_HIGHLIGHT_SIZE && !args.force_color;
        let exceeds_height = util::file_exceeds_terminal_height(file_path).unwrap_or(false);

        let use_pager = match args.pager {
            PagerMode::Always => true,
            PagerMode::Never | PagerMode::No => false,
            PagerMode::Auto => is_tty && exceeds_height,
        };

        let script_ref = get_lua_script();
        let info = get_nvim_info(args.theme.clone());

        if use_pager {
            run_tui_pager(file_path, &script_ref, args.force_color, &info)?;
        } else {
            if multiple_files {
                let mut out = io::stdout().lock();
                if i > 0 {
                    let _ = out.write_all(b"\n\n");
                }
                let _ = out.write_all(
                    format!("\x1b[1;34m:: {} ::\x1b[0m\n", file_path.display()).as_bytes(),
                );
            }

            if skip_highlight {
                let mut f = File::open(file_path)?;
                let mut out = io::stdout().lock();
                io::copy(&mut f, &mut out)?;
                continue;
            }

            let use_nuclear = size > CHUNK_THRESHOLD_BYTES;
            let result = if use_nuclear {
                process_large_file_to_stdout(file_path, &script_ref, args.force_color, &info)
            } else {
                let res = process_file_capture(file_path, &script_ref, args.force_color, &info);
                match res {
                    Ok(bytes) => {
                        let mut out = io::stdout().lock();
                        out.write_all(&bytes)?;
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            };
            if let Err(e) = result {
                if e.kind() != io::ErrorKind::BrokenPipe {
                    eprintln!("meow: {}", e);
                }
            }
        }
    }

    if let Some(path) = LUA_SCRIPT_PATH.get() {
        let _ = fs::remove_file(path.as_ref());
    }
    Ok(())
}

enum PagerMsg {
    Chunk(usize, Vec<String>),
    Error(String),
    Done,
}

fn run_tui_pager(
    path: &Path,
    script_path: &Path,
    force_color: bool,
    info: &NvimInfo,
) -> io::Result<()> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    let (tx, rx) = mpsc::channel();
    let path_buf = path.to_path_buf();
    let script_buf = script_path.to_path_buf();
    let info_clone = info.clone();

    thread::spawn(move || {
        let _ = load_file_parallel(&path_buf, &script_buf, force_color, &info_clone, tx);
    });

    let mut lines: Vec<String> = Vec::new();
    let mut chunks_buffer: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    let mut next_chunk_idx = 0;
    let mut scroll_y = 0;
    let mut term_size = terminal::size()?;
    let mut term_cols = term_size.0 as usize;
    let mut term_rows = term_size.1 as usize;
    let mut content_height = term_rows.saturating_sub(1);
    let mut finished_loading = false;
    let mut redraw = true;
    let mut spinner_idx = 0;
    let mut tick_count = 0;
    let mut gutter_width = 4;

    loop {
        let mut got_data = false;
        loop {
            match rx.try_recv() {
                Ok(PagerMsg::Chunk(idx, data)) => {
                    chunks_buffer.insert(idx, data);
                    while let Some(chunk) = chunks_buffer.remove(&next_chunk_idx) {
                        lines.extend(chunk);
                        next_chunk_idx += 1;
                        got_data = true;
                    }
                }
                Ok(PagerMsg::Error(e)) => {
                    lines.push(format!("\x1b[31mError: {}\x1b[0m", e));
                    finished_loading = true;
                    redraw = true;
                }
                Ok(PagerMsg::Done) => {
                    finished_loading = true;
                    redraw = true;
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    finished_loading = true;
                    break;
                }
            }
        }

        if got_data {
            redraw = true;
            let digits = (lines.len() as f64).log10().floor() as usize + 1;
            if digits > gutter_width {
                gutter_width = digits;
            }
        }

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            break;
                        }
                        KeyCode::Char('j') | KeyCode::Down | KeyCode::Enter => {
                            if scroll_y + content_height < lines.len() {
                                scroll_y += 1;
                                redraw = true;
                            }
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            if scroll_y > 0 {
                                scroll_y -= 1;
                                redraw = true;
                            }
                        }
                        KeyCode::PageDown | KeyCode::Char(' ') => {
                            scroll_y = (scroll_y + content_height)
                                .min(lines.len().saturating_sub(content_height));
                            redraw = true;
                        }
                        KeyCode::PageUp | KeyCode::Char('b') => {
                            scroll_y = scroll_y.saturating_sub(content_height);
                            redraw = true;
                        }
                        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            let half = content_height / 2;
                            scroll_y =
                                (scroll_y + half).min(lines.len().saturating_sub(content_height));
                            redraw = true;
                        }
                        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            let half = content_height / 2;
                            scroll_y = scroll_y.saturating_sub(half);
                            redraw = true;
                        }
                        KeyCode::Home | KeyCode::Char('g') => {
                            scroll_y = 0;
                            redraw = true;
                        }
                        KeyCode::End | KeyCode::Char('G') => {
                            if !lines.is_empty() {
                                scroll_y = lines.len().saturating_sub(content_height);
                            }
                            redraw = true;
                        }
                        _ => {}
                    }
                }
            }
        }

        let new_size = terminal::size()?;
        if new_size.1 as usize != term_rows || new_size.0 as usize != term_cols {
            term_size = new_size;
            term_cols = term_size.0 as usize;
            term_rows = term_size.1 as usize;
            content_height = term_rows.saturating_sub(1);
            redraw = true;
        }

        if !finished_loading {
            tick_count += 1;
            if tick_count % 5 == 0 {
                spinner_idx = (spinner_idx + 1) % 4;
                redraw = true;
            }
        }

        if redraw {
            queue!(
                stdout,
                terminal::Clear(ClearType::All),
                cursor::MoveTo(0, 0)
            )?;
            let end_line = (scroll_y + content_height).min(lines.len());
            for i in scroll_y..end_line {
                queue!(
                    stdout,
                    SetForegroundColor(Color::DarkGrey),
                    Print(format!("{:>width$} │ ", i + 1, width = gutter_width)),
                    ResetColor,
                    Print(&lines[i]),
                    Print("\r\n")
                )?;
            }
            queue!(
                stdout,
                cursor::MoveTo(0, term_rows as u16 - 1),
                SetAttribute(Attribute::Reverse)
            )?;
            let filename = path.display().to_string();
            let spinner = if finished_loading {
                ""
            } else {
                match spinner_idx {
                    0 => "|",
                    1 => "/",
                    2 => "-",
                    _ => "\\",
                }
            };
            let percentage = if lines.is_empty() {
                0
            } else {
                (scroll_y * 100) / lines.len()
            };
            let pos_info = format!(
                " {}:{} | {}% {} ",
                scroll_y + 1,
                lines.len(),
                percentage,
                spinner
            );
            let status_left = format!(" {} ", filename);
            let padding_len = term_cols.saturating_sub(status_left.len() + pos_info.len());
            queue!(
                stdout,
                Print(status_left),
                Print(" ".repeat(padding_len)),
                Print(pos_info),
                ResetColor
            )?;
            stdout.flush()?;
            redraw = false;
        }
    }
    execute!(stdout, cursor::Show, LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

fn load_file_parallel(
    path: &Path,
    script_path: &Path,
    force_color: bool,
    info: &NvimInfo,
    tx: mpsc::Sender<PagerMsg>,
) -> io::Result<()> {
    let file = File::open(path)?;
    let size = file.metadata()?.len();

    if util::is_binary_or_device(path).unwrap_or(false) {
        let _ = tx.send(PagerMsg::Error("Binary/Device detected".into()));
        let _ = tx.send(PagerMsg::Done);
        return Ok(());
    }

    if size > MAX_HIGHLIGHT_SIZE && !force_color {
        let reader = BufReader::new(file);
        let mut chunk = Vec::new();
        for line in reader.lines() {
            if let Ok(l) = line {
                chunk.push(l);
            }
            if chunk.len() >= 1000 {
                let _ = tx.send(PagerMsg::Chunk(0, chunk));
                chunk = Vec::new();
            }
        }
        if !chunk.is_empty() {
            let _ = tx.send(PagerMsg::Chunk(0, chunk));
        }
        let _ = tx.send(PagerMsg::Done);
        return Ok(());
    }

    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let chunk_approx_size = size / num_threads as u64;
    let reader = BufReader::new(file);
    let mut temp_files = Vec::new();
    let ext = path.extension().unwrap_or_default();
    let mut current_chunk_size = 0;
    let mut chunk_idx = 0;

    let mut temp_path = std::env::temp_dir();
    temp_path.push(format!(
        "meow_chunk_{}_{}.{}",
        std::process::id(),
        chunk_idx,
        ext.to_string_lossy()
    ));
    let mut current_writer = File::create(&temp_path)?;
    temp_files.push(temp_path.clone());

    for line in reader.lines() {
        let line = line?;
        let bytes = line.as_bytes();
        current_writer.write_all(bytes)?;
        current_writer.write_all(b"\n")?;
        current_chunk_size += bytes.len() as u64 + 1;
        if current_chunk_size >= chunk_approx_size && chunk_idx < num_threads - 1 {
            chunk_idx += 1;
            current_chunk_size = 0;
            temp_path = std::env::temp_dir();
            temp_path.push(format!(
                "meow_chunk_{}_{}.{}",
                std::process::id(),
                chunk_idx,
                ext.to_string_lossy()
            ));
            current_writer = File::create(&temp_path)?;
            temp_files.push(temp_path.clone());
        }
    }
    drop(current_writer);

    let mut handles = Vec::new();
    for (i, tfp) in temp_files.into_iter().enumerate() {
        let script = script_path.to_path_buf();
        let t_info = info.clone();
        let t_path = tfp.clone();
        let thread_tx = tx.clone();
        handles.push(thread::spawn(move || {
            let res = process_file_capture(&t_path, &script, force_color, &t_info);
            let _ = fs::remove_file(&t_path);
            match res {
                Ok(bytes) => {
                    let lines: Vec<String> = String::from_utf8_lossy(&bytes)
                        .lines()
                        .map(|l| l.to_string())
                        .collect();
                    let _ = thread_tx.send(PagerMsg::Chunk(i, lines));
                }
                Err(e) => {
                    let _ = thread_tx.send(PagerMsg::Error(e.to_string()));
                }
            }
        }));
    }
    for h in handles {
        let _ = h.join();
    }
    let _ = tx.send(PagerMsg::Done);
    Ok(())
}

fn process_large_file_to_stdout(
    path: &Path,
    script_path: &Path,
    force_color: bool,
    info: &NvimInfo,
) -> io::Result<()> {
    let file = File::open(path)?;
    let size = file.metadata()?.len();
    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let chunk_approx_size = size / num_threads as u64;
    let reader = BufReader::new(file);
    let mut temp_files = Vec::new();
    let ext = path.extension().unwrap_or_default();
    let mut current_chunk_size = 0;
    let mut chunk_idx = 0;

    let mut temp_path = std::env::temp_dir();
    temp_path.push(format!(
        "meow_pipe_{}_{}.{}",
        std::process::id(),
        chunk_idx,
        ext.to_string_lossy()
    ));
    let mut current_writer = File::create(&temp_path)?;
    temp_files.push(temp_path.clone());

    for line in reader.lines() {
        let line = line?;
        let bytes = line.as_bytes();
        current_writer.write_all(bytes)?;
        current_writer.write_all(b"\n")?;
        current_chunk_size += bytes.len() as u64 + 1;
        if current_chunk_size >= chunk_approx_size && chunk_idx < num_threads - 1 {
            chunk_idx += 1;
            current_chunk_size = 0;
            temp_path = std::env::temp_dir();
            temp_path.push(format!(
                "meow_pipe_{}_{}.{}",
                std::process::id(),
                chunk_idx,
                ext.to_string_lossy()
            ));
            current_writer = File::create(&temp_path)?;
            temp_files.push(temp_path.clone());
        }
    }
    drop(current_writer);

    let mut handles = Vec::new();
    for (i, tfp) in temp_files.into_iter().enumerate() {
        let script = script_path.to_path_buf();
        let t_info = info.clone();
        let t_path = tfp.clone();
        handles.push((
            i,
            thread::spawn(move || {
                let res = process_file_capture(&t_path, &script, force_color, &t_info);
                let _ = fs::remove_file(&t_path);
                res
            }),
        ));
    }
    handles.sort_by_key(|k| k.0);
    let mut out = io::stdout().lock();
    for (_, h) in handles {
        if let Ok(Ok(data)) = h.join() {
            out.write_all(&data)?;
        }
    }
    Ok(())
}

fn get_neovim_info_safe() -> io::Result<NvimInfo> {
    let script = r#"vim.schedule(function() io.write('THEME:'..(vim.g.colors_name or 'habamax')..'\n') io.write('RTP:'..vim.o.runtimepath..'\n') vim.cmd('qa!') end)"#;
    let mut child = Command::new("nvim")
        .arg("--headless")
        .args([
            "-c",
            "set eventignore+=VimEnter,UIEnter shortmess+=I nomore",
        ])
        .args(["-c", &format!("lua {}", script)])
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"\n\n\n\n");
    }
    for _ in 0..30 {
        if let Ok(Some(_)) = child.try_wait() {
            let output = child.wait_with_output()?;
            let raw = String::from_utf8_lossy(&output.stdout);
            let mut theme = "habamax".to_string();
            let mut rtp = "".to_string();
            for line in raw.lines() {
                if line.starts_with("THEME:") {
                    theme = line.replace("THEME:", "").trim().to_string();
                } else if line.starts_with("RTP:") {
                    rtp = line.replace("RTP:", "").trim().to_string();
                }
            }
            if theme.is_empty() || theme == "nil" {
                theme = "habamax".to_string();
            }
            return Ok(NvimInfo { theme, rtp });
        }
        thread::sleep(Duration::from_millis(50));
    }
    let _ = child.kill();
    Err(io::Error::new(
        io::ErrorKind::TimedOut,
        "Neovim config load timed out",
    ))
}

fn process_file_capture(
    path: &Path,
    script_path: &Path,
    _force_color: bool,
    info: &NvimInfo,
) -> io::Result<Vec<u8>> {
    if !path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "No such file"));
    }
    let path_str = path
        .canonicalize()?
        .to_str()
        .ok_or(io::Error::new(io::ErrorKind::InvalidData, "Invalid Path"))?
        .to_string();
    let lua_cmd = format!("luafile {}", script_path.display());
    let mut child = Command::new("nvim")
        .arg("--headless")
        .args(["--noplugin", "-c", "set shortmess+=I nomore"])
        .arg(&path_str)
        .env("MEOW_THEME", &info.theme)
        .env("MEOW_RTP", &info.rtp)
        .args(["-c", &lua_cmd])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"\n\n\n\n");
    }
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "Neovim error"));
    }
    Ok(output.stdout)
}
