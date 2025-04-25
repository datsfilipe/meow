use crate::util;

use nvim_rs::{
    Neovim, compat::tokio::Compat, create::tokio as create, rpc::handler::Dummy as DummyHandler,
};
use tokio::process::{ChildStdin, Command};

pub struct Nvim {
    instance: Neovim<Compat<ChildStdin>>,
}

impl Nvim {
    pub async fn new(config_path: &str) -> Self {
        let handler = DummyHandler::new();

        let path = util::path::get_nvim_bin_path();
        let (nvim, _io_handle, _child) = create::new_child_cmd(
            Command::new(path)
                .args(&["--embed", "--headless"])
                .env("NVIM_LOG_FILE", "nvimlog")
                .env("NVIM_APPNAME", "nv-meow")
                .env("XDG_CONFIG_HOME", config_path),
            handler,
        )
        .await
        .unwrap();

        Self { instance: nvim }
    }

    pub async fn hello_world(&self) {
        let buf = self.instance.create_buf(true, true).await.unwrap();
        self.instance.set_current_buf(&buf).await.unwrap();
        self.instance
            .set_current_line(&"Hello World!".to_string())
            .await
            .unwrap();
        let line = self.instance.get_current_line().await.unwrap();
        println!("{}", line);
    }
}
