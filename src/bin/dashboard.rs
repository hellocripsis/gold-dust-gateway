use std::error::Error;
use std::fs;
use std::net::SocketAddr;

use axum::{
    http::StatusCode,
    response::{Html, Redirect},
    routing::{get, post},
    Router,
};
use gold_dust_gateway::FLAG_PATH;
use tokio::net::TcpListener;

fn read_flag() -> bool {
    match fs::read_to_string(FLAG_PATH) {
        Ok(s) => s.trim() == "on",
        Err(_) => true, // default ON
    }
}

fn write_flag(on: bool) -> std::io::Result<()> {
    fs::write(FLAG_PATH, if on { "on\n" } else { "off\n" })
}

fn render_index(on: bool) -> String {
    let status_text = if on { "ON (Tor)" } else { "OFF (Direct)" };
    let status_class = if on { "on" } else { "off" };
    let button_label = if on {
        "Switch to Direct (OFF)"
    } else {
        "Switch to Tor (ON)"
    };
    let button_action = if on { "/off" } else { "/on" };

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title>Gold Dust Gateway Control</title>
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
    form {{
      margin: 0;
    }}
    button {{
      display: inline-block;
      margin-top: 8px;
      padding: 10px 22px;
      border-radius: 999px;
      font-weight: 600;
      letter-spacing: 0.04em;
      text-transform: uppercase;
      font-size: 0.85rem;
      background: #f1c40f;
      color: #11131a;
      border: none;
      cursor: pointer;
    }}
    button:hover {{
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
    <h1>Gold Dust Gateway</h1>
    <div class="status">
      Proxy status:
      <span class="{status_class}">{status_text}</span>
    </div>
    <form method="post" action="{button_action}">
      <button type="submit">{button_label}</button>
    </form>
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
        button_action = button_action,
        button_label = button_label
    )
}

fn render_error(msg: &str) -> Html<String> {
    Html(format!(
        r#"<!doctype html>
<html lang="en">
<head><meta charset="utf-8" /><title>Gold Dust Dashboard Error</title></head>
<body style="font-family: sans-serif; background:#111; color:#eee; padding: 2rem;">
  <h1>Dashboard Error</h1>
  <p>{}</p>
  <p><a href="/" style="color:#f1c40f;">Back to dashboard</a></p>
</body>
</html>"#,
        msg
    ))
}

async fn index() -> Html<String> {
    Html(render_index(read_flag()))
}

async fn set_on() -> Result<Redirect, (StatusCode, Html<String>)> {
    write_flag(true).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            render_error(&format!("Failed to switch Tor mode on: {e}")),
        )
    })?;
    Ok(Redirect::to("/"))
}

async fn set_off() -> Result<Redirect, (StatusCode, Html<String>)> {
    write_flag(false).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            render_error(&format!("Failed to switch direct mode off: {e}")),
        )
    })?;
    Ok(Redirect::to("/"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let app = Router::new()
        .route("/", get(index))
        .route("/on", post(set_on))
        .route("/off", post(set_off));

    let addr: SocketAddr = "127.0.0.1:3000".parse()?;
    let listener = TcpListener::bind(addr).await?;

    println!("[dashboard] Web UI listening on http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
