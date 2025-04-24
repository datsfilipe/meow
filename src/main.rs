mod nvim;
mod util;

#[tokio::main]
async fn main() {
    match util::validate::is_nvim_installed() {
        true => {}
        false => {
            println!("nvim not found, nyah!");
            return;
        }
    }

    let config = nvim::config::Config::new(None::<std::path::PathBuf>);
    config.init_default().unwrap();

    let nvim = nvim::handler::Nvim::new(config.path.to_str().unwrap()).await;
    nvim.hello_world().await;
}
