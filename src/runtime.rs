use std::{future::Future, io, pin::Pin, time::Duration};

use tokio::{net::TcpStream, time};

pub type ReadinessFuture = Pin<Box<dyn Future<Output = io::Result<()>> + Send>>;

pub trait ReadinessProbe: Send + Sync {
    fn wait_until_ready(&self, host_port: u16, timeout: Duration) -> ReadinessFuture;
}

#[derive(Debug)]
pub struct TcpReadinessProbe {
    host: String,
}

impl TcpReadinessProbe {
    pub fn new(host: String) -> Self {
        Self { host }
    }
}

impl ReadinessProbe for TcpReadinessProbe {
    fn wait_until_ready(&self, host_port: u16, timeout: Duration) -> ReadinessFuture {
        let host = self.host.clone();
        Box::pin(async move {
            let deadline = time::Instant::now() + timeout;

            loop {
                if TcpStream::connect((host.as_str(), host_port)).await.is_ok() {
                    return Ok(());
                }

                if time::Instant::now() >= deadline {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        format!("application port {host_port} was not ready within {timeout:?}"),
                    ));
                }

                time::sleep(Duration::from_millis(100)).await;
            }
        })
    }
}
