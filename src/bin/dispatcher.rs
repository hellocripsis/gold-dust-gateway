use std::error::Error;
use std::fs;
use std::net::SocketAddr;

use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_socks::tcp::Socks5Stream;

const FLAG_PATH: &str = "gold-dust-tor.flag";

fn should_use_tor() -> bool {
    match fs::read_to_string(FLAG_PATH) {
        Ok(s) => s.trim() == "on",
        Err(_) => true, // default: ON if file missing
    }
}

async fn handle_client(mut inbound: TcpStream) -> Result<(), Box<dyn Error + Send + Sync>> {
    // 1) Read HTTP CONNECT request header
    let mut buf = Vec::with_capacity(1024);
    loop {
        let mut byte = [0u8; 1];
        let n = inbound.read(&mut byte).await?;
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

    let target = target.to_string();

    if should_use_tor() {
        // 2a) VIA TOR (SOCKS5 → 127.0.0.1:9050)
        let mut outbound = Socks5Stream::connect("127.0.0.1:9050", target.clone()).await?;
        inbound
            .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
            .await?;

        io::copy_bidirectional(&mut inbound, &mut outbound).await?;
    } else {
        // 2b) DIRECT TCP
        let mut outbound = TcpStream::connect(target.clone()).await?;
        inbound
            .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
            .await?;

        io::copy_bidirectional(&mut inbound, &mut outbound).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
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
