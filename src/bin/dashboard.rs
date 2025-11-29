use std::fs;
use std::net::SocketAddr;

use axum::{
    response::{Html, Redirect},
    routing::get,
    Router,
};
use tokio::net::TcpListener;

const FLAG_PATH: &str = "gold-dust-tor.flag";

fn read_flag() -> bool {
    match fs::read_to_string(FLAG_PATH) {
        Ok(s) => s.trim() == "on",
        Err(_) => true, // default ON
    }
}

fn write_flag(on: bool) {
    let _ = fs::write(FLAG_PATH, if on { "on\n" } else { "off\n" });
}

async fn index() -> Html<String> {
    let on = read_flag();
    let status_text = if on { "ON (Tor)" } else { "OFF (Direct)" };
    let status_class = if on { "on" } else { "off" };
    let button_label = if on {
        "Switch to Direct (OFF)"
    } else {
        "Switch to Tor (ON)"
    };
    let button_href = if on { "/off" } else { "/on" };

    let html = format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title>Gold Dust VPN Control</title>
  <style>
    body {{
      background: #0b0c10;
      color: #e5e5e5;
      font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      display: flex;
      align-items: center;
      justify-content: center;
      height: 100vh;
      margin: 0;
    }}
    .card {{
      background: #11131a;
      border-radius: 16px;
      padding: 32px 40px;
      box-shadow: 0 18px 45px rgba(0,0,0,0.6);
      text-align: center;
      min-width: 360px;
      border: 1px solid #222837;
    }}
    h1 {{
      margin-top: 0;
      margin-bottom: 8px;
      font-size: 1.5rem;
      letter-spacing: 0.06em;
      text-transform: uppercase;
      color: #f1c40f;
    }}
    .status {{
      margin-bottom: 24px;
      font-size: 1.1rem;
    }}
    .status span {{
      font-weight: 600;
      padding: 4px 10px;
      border-radius: 999px;
    }}
    .status .on {{
      background: rgba(39, 174, 96, 0.15);
      color: #2ecc71;
      border: 1px solid rgba(46, 204, 113, 0.6);
    }}
    .status .off {{
      background: rgba(231, 76, 60, 0.15);
      color: #e74c3c;
      border: 1px solid rgba(231, 76, 60, 0.6);
    }}
    a.button {{
      display: inline-block;
      margin-top: 8px;
      padding: 10px 22px;
      border-radius: 999px;
      text-decoration: none;
      font-weight: 600;
      letter-spacing: 0.04em;
      text-transform: uppercase;
      font-size: 0.85rem;
      background: #f1c40f;
      color: #11131a;
      border: none;
    }}
    a.button:hover {{
      background: #f5d76e;
    }}
    .hint {{
      margin-top: 18px;
      font-size: 0.8rem;
      color: #9ca3af;
    }}
    code {{
      background: #1f2937;
      padding: 2px 6px;
      border-radius: 4px;
      font-size: 0.78rem;
    }}
  </style>
</head>
<body>
  <div class="card">
    <h1>Gold Dust VPN</h1>
    <div class="status">
      Proxy status:
      <span class="{status_class}">{status_text}</span>
    </div>
    <a class="button" href="{button_href}">{button_label}</a>
    <div class="hint">
      Browser HTTP proxy: <code>127.0.0.1:7777</code><br/>
      This only affects apps pointed at the Gold Dust proxy.
    </div>
  </div>
</body>
</html>
"#,
        status_class = status_class,
        status_text = status_text,
        button_href = button_href,
        button_label = button_label
    );

    Html(html)
}

async fn set_on() -> Redirect {
    write_flag(true);
    Redirect::to("/")
}

async fn set_off() -> Redirect {
    write_flag(false);
    Redirect::to("/")
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/on", get(set_on))
        .route("/off", get(set_off));

    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();

    println!("[dashboard] Web UI listening on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}
