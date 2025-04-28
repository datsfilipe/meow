use std::{env, path::PathBuf};

#[derive(Debug)]
pub struct Args {
    pub file_path: Option<String>,
    pub config_path: Option<PathBuf>,
    pub add_colorscheme: Option<String>,
    pub set_colorscheme: Option<String>,
    pub remove_colorscheme: Option<String>,
    pub help: Option<bool>,
    pub version: Option<bool>,
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
        version: None,
    };

    let mut expecting_config_path = false;
    let mut seen_colorscheme_command = false;

    let home = env::var_os("HOME");
    while let Some(arg) = args_iter.next() {
        if expecting_config_path {
            let expanded_path = if arg.starts_with("~/") {
                if let Some(home_val) = &home {
                    let mut path_buf = PathBuf::from(home_val);
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
                if let Some(home_val) = &home {
                    let mut path_buf = PathBuf::from(home_val);
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
                    return Err("colorscheme commands are not composable, nya!".into());
                }

                let value = arg
                    .strip_prefix("--add-colorscheme=")
                    .ok_or_else(|| "invalid format for --add-colorscheme, nya!".to_string())?
                    .to_string();

                result.add_colorscheme = Some(value);
                seen_colorscheme_command = true;
            }
            "--add-colorscheme" => {
                if seen_colorscheme_command {
                    return Err("colorscheme commands are not composable, nya!".into());
                }

                let value = args_iter
                    .next()
                    .ok_or_else(|| "missing value for --add-colorscheme, nya!".to_string())?;

                result.add_colorscheme = Some(value);
                seen_colorscheme_command = true;
            }
            arg if arg.starts_with("--set-colorscheme=") => {
                if seen_colorscheme_command {
                    return Err("colorscheme commands are not composable, nya!".into());
                }

                let value = arg
                    .strip_prefix("--set-colorscheme=")
                    .ok_or_else(|| "Invalid format for --set-colorscheme".to_string())?
                    .to_string();

                result.set_colorscheme = Some(value);
                seen_colorscheme_command = true;
            }
            "--set-colorscheme" => {
                if seen_colorscheme_command {
                    return Err("Colorscheme commands are not composable".into());
                }

                let value = args_iter
                    .next()
                    .ok_or_else(|| "missing value for --set-colorscheme, nya!".to_string())?;

                result.set_colorscheme = Some(value);
                seen_colorscheme_command = true;
            }
            arg if arg.starts_with("--remove-colorscheme=") => {
                if seen_colorscheme_command {
                    return Err("colorscheme commands are not composable, nya!".into());
                }

                let value = arg
                    .strip_prefix("--remove-colorscheme=")
                    .ok_or_else(|| "Invalid format for --remove-colorscheme".to_string())?
                    .to_string();

                result.remove_colorscheme = Some(value);
                seen_colorscheme_command = true;
            }
            "--remove-colorscheme" => {
                if seen_colorscheme_command {
                    return Err("colorscheme commands are not composable, nya!".into());
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
            "--version" => {
                result.version = Some(true);
            }
            _ => {
                if seen_colorscheme_command {
                    return Err("file arguments not allowed with colorscheme commands, nya!".into());
                }

                if result.file_path.is_some() {
                    return Err("only one file path is allowed, nya!".into());
                }

                result.file_path = Some(arg);
            }
        }
    }

    if expecting_config_path {
        return Err("missing path for --config, nya!".into());
    }

    Ok(result)
}

pub fn print_usage() {
    eprintln!("usage:");
    eprintln!("  meow [FILE]");
    eprintln!("  meow --config PATH [FILE]");
    eprintln!("  meow --add-colorscheme USER/REPO(/TREE/BRANCH)");
    eprintln!("  meow --set-colorscheme USER/REPO");
    eprintln!("  meow --remove-colorscheme USER/REPO");
    eprintln!();
    eprintln!(
        "note: colorscheme commands cannot be combined with each other or with file arguments, nya!"
    );
}

pub fn print_version() {
    let version = env!("CARGO_PKG_VERSION");
    eprintln!("meow {}", version);
}
