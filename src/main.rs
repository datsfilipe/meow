mod nvim;

#[tokio::main]
async fn main() {
    let nvim = nvim::handler::Nvim::new().await;
    nvim.hello_world().await;
}
