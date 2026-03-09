use crate::models::HookEvent;
use tokio::sync::mpsc;

#[cfg(unix)]
use tokio::net::UnixListener;

#[cfg(unix)]
const SOCKET_PATH: &str = "/tmp/clawbit.sock";

pub struct IpcServer {
    event_tx: mpsc::UnboundedSender<HookEvent>,
}

impl IpcServer {
    pub fn new(event_tx: mpsc::UnboundedSender<HookEvent>) -> Self {
        Self { event_tx }
    }

    #[cfg(unix)]
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let _ = std::fs::remove_file(SOCKET_PATH);
        let listener = UnixListener::bind(SOCKET_PATH)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(SOCKET_PATH, std::fs::Permissions::from_mode(0o666))?;
        }

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let tx = self.event_tx.clone();
                    tokio::spawn(async move {
                        Self::handle_connection(stream, tx).await;
                    });
                }
                Err(e) => eprintln!("IPC accept error: {}", e),
            }
        }
    }

    #[cfg(unix)]
    async fn handle_connection(
        stream: tokio::net::UnixStream,
        tx: mpsc::UnboundedSender<HookEvent>,
    ) {
        use tokio::io::AsyncReadExt;
        let mut stream = stream;
        let mut buf = Vec::new();
        if stream.read_to_end(&mut buf).await.is_ok() {
            if let Ok(text) = String::from_utf8(buf) {
                for line in text.lines() {
                    if let Ok(event) = serde_json::from_str::<HookEvent>(line) {
                        let _ = tx.send(event);
                    }
                }
            }
        }
    }

    #[cfg(windows)]
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        use tokio::io::AsyncReadExt;
        use tokio::net::windows::named_pipe::ServerOptions;

        let pipe_name = r"\\.\pipe\clawbit";
        loop {
            let server = ServerOptions::new().create(pipe_name)?;
            server.connect().await?;
            let tx = self.event_tx.clone();
            tokio::spawn(async move {
                let mut server = server;
                let mut buf = vec![0u8; 8192];
                if let Ok(n) = server.read(&mut buf).await {
                    if let Ok(text) = std::str::from_utf8(&buf[..n]) {
                        for line in text.lines() {
                            if let Ok(event) = serde_json::from_str::<HookEvent>(line) {
                                let _ = tx.send(event);
                            }
                        }
                    }
                }
            });
        }
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        #[cfg(unix)]
        let _ = std::fs::remove_file(SOCKET_PATH);
    }
}
