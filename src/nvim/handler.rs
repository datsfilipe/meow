use crate::util;

use nvim_rs::{Neovim, Value, compat::tokio::Compat, create::tokio as create, error::LoopError};
use std::path::Path;
use tokio::process::ChildStdin;

pub struct Nvim {
    instance: Neovim<Compat<ChildStdin>>,
    _io: tokio::task::JoinHandle<Result<(), Box<LoopError>>>,
    _child: tokio::process::Child,
}

impl Drop for Nvim {
    fn drop(&mut self) {
        self._io.abort();
        let _ = self._child.start_kill();
    }
}

impl Nvim {
    pub async fn new(config: &str) -> Self {
        let (n, io, c) = create::new_child_cmd(
            tokio::process::Command::new(util::path::get_nvim_bin_path())
                .args(&["--embed", "--headless"])
                .env("XDG_CONFIG_HOME", config),
            nvim_rs::rpc::handler::Dummy::new(),
        )
        .await
        .unwrap();
        Self {
            instance: n,
            _io: io,
            _child: c,
        }
    }

    pub async fn print_file_with_highlighting(
        &self,
        file_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let abs = Path::new(file_path).canonicalize()?;
        let path = abs.to_str().ok_or("invalid UTF-8 in path")?;
        let mut del = None;

        let res = async {
            let buf = self.instance.create_buf(false, true).await.unwrap();
            let num = buf.get_number().await?;
            del = Some(num);
            self.instance.set_current_buf(&buf).await?;

            let esc = path.replace(' ', r"\ ");
            self.instance.command(&format!("edit {}", esc)).await?;

            let val = self
                .instance
                .call_function(
                    "nvim_buf_get_lines",
                    vec![
                        Value::from(0),
                        Value::from(0),
                        Value::from(-1),
                        Value::from(true),
                    ],
                )
                .await?;

            if let Value::Array(a) = val {
                if a.is_empty() {
                    let cnt = self
                        .instance
                        .call_function("line", vec![Value::from("$")])
                        .await?
                        .as_i64()
                        .unwrap_or(0);
                    if cnt > 0 {
                        eprintln!("Warning: highlights not ready.");
                    }
                }
                for v in a {
                    if let Some(l) = v.as_str() {
                        println!("{}", l)
                    }
                }
            }
            Ok::<(), Box<dyn std::error::Error>>(())
        }
        .await;

        if let Some(n) = del {
            let _ = self
                .instance
                .command(&format!("silent! bdelete! {}", n))
                .await;
        }
        res.map_err(|e| {
            eprintln!("{}", e);
            std::process::exit(1)
        })
    }
}
