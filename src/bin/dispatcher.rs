use std::error::Error;
use std::fs;
use std::net::SocketAddr;

use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{timeout, Duration};
use tokio_socks::tcp::Socks5Stream;

const FLAG_PATH: &str = "gold-dust-tor.flag";
const HEADER_READ_TIMEOUT: Duration = Duration::from_secs(10);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const TUNNEL_IDLE_TIMEOUT: Duration = Duration::from_secs(300);

type DynError = Box<dyn Error + Send + Sync>;

fn should_use_tor() -> bool {
    match fs::read_to_string(FLAG_PATH) {
        Ok(s) => s.trim() == "on",
        Err(_) => true, // default: ON if file missing
    }
}

fn validate_connect_target(target: &str) -> Result<(), &'static str> {
    let (host, port) = target.rsplit_once(':').ok_or("target must be host:port")?;
    if host.trim().is_empty() {
        return Err("target host is empty");
    }
    let parsed_port: u16 = port.parse().map_err(|_| "target port is invalid")?;
    if parsed_port == 0 {
        return Err("target port must be > 0");
    }
    Ok(())
}

async fn handle_client(mut inbound: TcpStream) -> Result<(), DynError> {
    // 1) Read HTTP CONNECT request header
    let mut buf = Vec::with_capacity(1024);
    loop {
        let mut byte = [0u8; 1];
        let n = timeout(HEADER_READ_TIMEOUT, inbound.read(&mut byte))
            .await
            .map_err(|_| "timed out while reading request header")??;
        if n == 0 {
            return Err("client closed before sending request".into());
        }
        buf.push(byte[0]);
        let len = buf.len();
        if len >= 4 && &buf[len - 4..] == b"\r\n\r\n" {
            break;
        }
        if buf.len() > 8192 {
            return Err("request header too large".into());
        }
    }

    let req = String::from_utf8_lossy(&buf);
    let mut lines = req.lines();
    let first = lines.next().ok_or("empty request")?;
    let mut parts = first.split_whitespace();
    let method = parts.next().unwrap_or("");
    let target = parts.next().unwrap_or("");
    let _version = parts.next().unwrap_or("");

    if method != "CONNECT" {
        let resp = b"HTTP/1.1 405 Method Not Allowed\r\n\r\n";
        inbound.write_all(resp).await?;
        return Ok(());
    }

    if let Err(msg) = validate_connect_target(target) {
        let resp = format!("HTTP/1.1 400 Bad Request\r\n\r\n{msg}\r\n");
        inbound.write_all(resp.as_bytes()).await?;
        return Ok(());
    }

    let target = target.to_owned();

    if should_use_tor() {
        // 2a) VIA TOR (SOCKS5 → 127.0.0.1:9050)
        let mut outbound = timeout(
            CONNECT_TIMEOUT,
            Socks5Stream::connect("127.0.0.1:9050", target.clone()),
        )
        .await
        .map_err(|_| "timed out connecting to Tor SOCKS5")??;
        inbound
            .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
            .await?;

        timeout(
            TUNNEL_IDLE_TIMEOUT,
            io::copy_bidirectional(&mut inbound, &mut outbound),
        )
        .await
        .map_err(|_| "tunnel idle timeout reached")??;
    } else {
        // 2b) DIRECT TCP
        let mut outbound = timeout(CONNECT_TIMEOUT, TcpStream::connect(target.clone()))
            .await
            .map_err(|_| "timed out connecting to target")??;
        inbound
            .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
            .await?;

        timeout(
            TUNNEL_IDLE_TIMEOUT,
            io::copy_bidirectional(&mut inbound, &mut outbound),
        )
        .await
        .map_err(|_| "tunnel idle timeout reached")??;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), DynError> {
    let addr: SocketAddr = "127.0.0.1:7777".parse()?;
    let listener = TcpListener::bind(addr).await?;
    println!(
        "[dispatcher] HTTP CONNECT proxy on {} (flag: {}, 'on' = Tor, 'off' = direct)",
        addr, FLAG_PATH
    );

    loop {
        let (socket, peer) = listener.accept().await?;
        println!("[dispatcher] new client from {}", peer);
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket).await {
                eprintln!("[dispatcher] error: {}", e);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::validate_connect_target;

    #[test]
    fn validates_standard_target() {
        assert!(validate_connect_target("example.com:443").is_ok());
    }

    #[test]
    fn rejects_target_without_port() {
        assert!(validate_connect_target("example.com").is_err());
    }

    #[test]
    fn rejects_target_with_invalid_port() {
        assert!(validate_connect_target("example.com:99999").is_err());
    }
}
