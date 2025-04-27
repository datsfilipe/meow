mod cli;
mod nvim;
mod util;

use std::sync::Arc;

#[tokio::main]
async fn main() {
    if !util::validate::is_nvim_installed() {
        println!("nvim not found, nya!");
        return;
    }

    match cli::args::parse_args() {
        Ok(args) => {
            if args.help.is_some() {
                cli::args::print_usage();
                return;
            }

            let config = nvim::config::Config::new(args.config_path);
            if let Some(repo) = args.add_colorscheme {
                if let Err(e) = config.add_colorscheme(&repo) {
                    eprintln!("Error adding colorscheme: {}, nya!", e);
                    std::process::exit(1);
                }
                return;
            }

            if let Some(repo) = args.set_colorscheme {
                if let Err(e) = config.set_colorscheme(&repo) {
                    eprintln!("Error setting colorscheme: {}, nya!", e);
                    std::process::exit(1);
                }
                return;
            }

            if let Some(repo) = args.remove_colorscheme {
                if let Err(e) = config.remove_colorscheme(&repo) {
                    eprintln!("Error removing colorscheme: {}, nya!", e);
                    std::process::exit(1);
                }
                return;
            }

            if let Some(file_path) = args.file_path {
                let (tx, mut rx) = tokio::sync::mpsc::channel::<
                    Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>,
                >(100);

                let config_path = Arc::new(config.path.to_string_lossy().to_string());
                let file_path_clone = file_path.clone();

                tokio::spawn(async move {
                    let nvim = nvim::handler::Nvim::new(&config_path).await;
                    if let Err(e) = nvim.print_file_with_highlighting(&file_path_clone).await {
                        let _ = tx.send(Err::<(), _>(e)).await;
                    } else {
                        let _ = tx.send(Ok(())).await;
                    }
                });

                if let Some(result) = rx.recv().await {
                    if let Err(e) = result {
                        eprintln!("Error highlighting file: {}, nya!", e);
                        std::process::exit(1);
                    }
                }
                return;
            }

            cli::args::print_usage();
        }
        Err(e) => {
            eprintln!("error parsing arguments: {}, nya!", e);
            cli::args::print_usage();
            std::process::exit(1);
        }
    }
}
