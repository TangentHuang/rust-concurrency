use anyhow::{bail, Result};
use log::warn;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tracing::info;

const BUF_SIZE: usize = 4096;
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let addr = "0.0.0.0:6379";
    let listener = TcpListener::bind(addr).await?;
    info!("[Dredis]listening on {}", addr);

    loop {
        let (stream, raddr) = listener.accept().await?;
        info!("[Dredis]accepted connection from {}", raddr);
        tokio::spawn(async move {
            process_redis_conn(stream, raddr).await?;
            Ok::<(), anyhow::Error>(())
        });
    }
}

async fn process_redis_conn(
    mut stream: tokio::net::TcpStream,
    raddr: std::net::SocketAddr,
) -> Result<()> {
    loop {
        stream.readable().await?;
        let mut buf = Vec::with_capacity(BUF_SIZE);
        match stream.try_read_buf(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                info!("[Dredis]read {} bytes", n);
                let line = String::from_utf8_lossy(&buf);
                info!("[Dredis]read data: {:?}", line);

                stream.write_all(b"+OK\r\n").await?;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
            Err(e) => {
                bail!("[Dredis]error: {}", e.to_string())
            }
        }
    }
    warn!("[Dredis]connection {} closed", raddr);
    Ok(())
}
