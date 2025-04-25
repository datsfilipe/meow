mod cli;
mod nvim;
mod util;

#[tokio::main]
async fn main() {
    match util::validate::is_nvim_installed() {
        true => {}
        false => {
            println!("nvim not found, nya!");
            return;
        }
    }

    let config = nvim::config::Config::new(None::<std::path::PathBuf>);
    let nvim = nvim::handler::Nvim::new(config.path.to_str().unwrap()).await;

    match cli::args::parse_args() {
        Ok(args) => {
            if args.help.is_some() {
                cli::args::print_usage();
                return;
            };

            if args.add_colorscheme.is_some() {
                config
                    .add_colorscheme(&args.add_colorscheme.unwrap())
                    .unwrap();
                return;
            }

            if args.set_colorscheme.is_some() {
                config
                    .set_colorscheme(&args.set_colorscheme.unwrap())
                    .unwrap();
                return;
            }

            if args.remove_colorscheme.is_some() {
                config
                    .remove_colorscheme(&args.remove_colorscheme.unwrap())
                    .unwrap();
                return;
            }

            if args.file_path.is_some() {
                nvim.print_file_with_highlighting(&args.file_path.unwrap())
                    .await
                    .unwrap();
                return;
            }

            if args.config_path.is_some() {
                if args.file_path.is_some() {
                    nvim.print_file_with_highlighting(&args.file_path.unwrap())
                        .await
                        .unwrap();
                    return;
                }

                println!("missing file path, nya!");
                std::process::exit(1);
            }

            cli::args::print_usage();
        }
        Err(e) => {
            eprintln!("error parsing arguments: {}, {}\n", e, "nya!");
            cli::args::print_usage();
            std::process::exit(1);
        }
    }
}
