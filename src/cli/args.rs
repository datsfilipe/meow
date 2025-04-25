use std::env;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Args {
    pub file_path: Option<String>,
    pub config_path: Option<PathBuf>,
    pub add_colorscheme: Option<String>,
    pub set_colorscheme: Option<String>,
    pub remove_colorscheme: Option<String>,
    pub help: Option<bool>,
}

pub fn parse_args() -> Result<Args, String> {
    let mut args_iter = env::args().skip(1);
    let mut result = Args {
        file_path: None,
        config_path: None,
        add_colorscheme: None,
        set_colorscheme: None,
        remove_colorscheme: None,
        help: None,
    };

    let mut expecting_config_path = false;
    let mut seen_colorscheme_command = false;

    while let Some(arg) = args_iter.next() {
        if expecting_config_path {
            let expanded_path = if arg.starts_with("~/") {
                if let Some(home) = env::var_os("HOME") {
                    let mut path_buf = PathBuf::from(home);
                    path_buf.push(&arg[2..]);
                    path_buf
                } else {
                    PathBuf::from(arg)
                }
            } else {
                PathBuf::from(arg)
            };

            result.config_path = Some(expanded_path);
            expecting_config_path = false;
            continue;
        }

        if let Some(config_path) = arg.strip_prefix("--config=") {
            let expanded_path = if config_path.starts_with("~/") {
                if let Some(home) = env::var_os("HOME") {
                    let mut path_buf = PathBuf::from(home);
                    path_buf.push(&config_path[2..]);
                    path_buf
                } else {
                    PathBuf::from(config_path)
                }
            } else {
                PathBuf::from(config_path)
            };

            result.config_path = Some(expanded_path);
            continue;
        }

        match arg.as_str() {
            "--config" => {
                expecting_config_path = true;
            }
            arg if arg.starts_with("--add-colorscheme=") => {
                if seen_colorscheme_command {
                    return Err("colorscheme commands are not composable, nya!".to_string());
                }

                let value = arg
                    .strip_prefix("--add-colorscheme=")
                    .map(|s| s.to_string())
                    .ok_or_else(|| "invalid format for --add-colorscheme, nya!".to_string())?;

                result.add_colorscheme = Some(value);
                seen_colorscheme_command = true;
            }
            "--add-colorscheme" => {
                if seen_colorscheme_command {
                    return Err("colorscheme commands are not composable, nya!".to_string());
                }

                let value = args_iter
                    .next()
                    .ok_or_else(|| "missing value for --add-colorscheme, nya!".to_string())?;

                result.add_colorscheme = Some(value);
                seen_colorscheme_command = true;
            }
            arg if arg.starts_with("--set-colorscheme=") => {
                if seen_colorscheme_command {
                    return Err("colorscheme commands are not composable, nya!".to_string());
                }

                let value = arg
                    .strip_prefix("--set-colorscheme=")
                    .map(|s| s.to_string())
                    .ok_or_else(|| "Invalid format for --set-colorscheme".to_string())?;

                result.set_colorscheme = Some(value);
                seen_colorscheme_command = true;
            }
            "--set-colorscheme" => {
                if seen_colorscheme_command {
                    return Err("Colorscheme commands are not composable".to_string());
                }

                let value = args_iter
                    .next()
                    .ok_or_else(|| "missing value for --set-colorscheme, nya!".to_string())?;

                result.set_colorscheme = Some(value);
                seen_colorscheme_command = true;
            }
            arg if arg.starts_with("--remove-colorscheme=") => {
                if seen_colorscheme_command {
                    return Err("colorscheme commands are not composable, nya!".to_string());
                }

                let value = arg
                    .strip_prefix("--remove-colorscheme=")
                    .map(|s| s.to_string())
                    .ok_or_else(|| "Invalid format for --remove-colorscheme".to_string())?;

                result.remove_colorscheme = Some(value);
                seen_colorscheme_command = true;
            }
            "--remove-colorscheme" => {
                if seen_colorscheme_command {
                    return Err("colorscheme commands are not composable, nya!".to_string());
                }

                let value = args_iter
                    .next()
                    .ok_or_else(|| "missing value for --remove-colorscheme, nya!".to_string())?;

                result.remove_colorscheme = Some(value);
                seen_colorscheme_command = true;
            }
            "--help" => {
                result.help = Some(true);
            }
            _ => {
                if seen_colorscheme_command {
                    return Err(
                        "file arguments not allowed with colorscheme commands, nya!".to_string()
                    );
                }

                if result.file_path.is_some() {
                    return Err("only one file path is allowed, nya!".to_string());
                }

                result.file_path = Some(arg);
            }
        }
    }

    if expecting_config_path {
        return Err("missing path for --config, nya!".to_string());
    }

    Ok(result)
}

pub fn print_usage() {
    eprintln!("usage:");
    eprintln!("  bin [FILE]");
    eprintln!("  bin --config PATH [FILE]");
    eprintln!("  bin --add-colorscheme USER/REPO(/TREE/BRANCH)");
    eprintln!("  bin --set-colorscheme USER/REPO");
    eprintln!("  bin --remove-colorscheme USER/REPO");
    eprintln!();
    eprintln!(
        "note: colorscheme commands cannot be combined with each other or with file arguments, nya!"
    );
}
