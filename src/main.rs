use nvim_rs::{create::tokio as create, rpc::handler::Dummy as DummyHandler};

use tokio::process::Command;

const NVIMPATH: &str = "/etc/profiles/per-user/dtsf/bin/nvim";

#[tokio::main]
async fn main() {
    let handler = DummyHandler::new();

    let (nvim, _io_handle, _child) = create::new_child_cmd(
        Command::new(NVIMPATH)
            .args(&["-u", "NONE", "--embed", "--headless"])
            .env("NVIM_LOG_FILE", "nvimlog"),
        handler,
    )
    .await
    .unwrap();

    let buf = nvim.create_buf(true, true).await.unwrap();
    nvim.set_current_buf(&buf).await.unwrap();
    nvim.set_current_line(&"Hello World!".to_string())
        .await
        .unwrap();
    let line = nvim.get_current_line().await.unwrap();
    println!("{}", line);
}
