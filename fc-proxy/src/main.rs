use axum::{
    Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Html,
    routing::get,
};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;

#[tokio::main]
async fn main() {
    // 1. Build the Web Server Router
    let app = Router::new()
        .route("/", get(serve_html))
        .route("/ws", get(ws_handler));

    // 2. Start listening on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Web Server running! Open http://localhost:3000 in your browser.");

    axum::serve(listener, app).await.unwrap();
}

// Serve the index.html file we created
async fn serve_html() -> Html<&'static str> {
    Html(include_str!("index.html"))
}

// This function runs every time a browser connects to the WebSocket
async fn ws_handler(ws: WebSocketUpgrade) -> axum::response::Response {
    ws.on_upgrade(handle_socket)
}

// The actual Orchestrator Logic
async fn handle_socket(mut socket: WebSocket) {
    println!("Browser connected! Spawning MicroVM...");
    let _ = std::fs::remove_file("/tmp/firecracker.socket");

    // Spawn Firecracker
    let mut child = Command::new("./firecracker")
        .current_dir("..")
        .arg("--api-sock")
        .arg("/tmp/firecracker.socket")
        .arg("--enable-pci")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn Firecracker");

    let mut child_stdin = child.stdin.take().unwrap();
    let mut child_stdout = child.stdout.take().unwrap();

    tokio::time::sleep(Duration::from_millis(500)).await;

    // API Configuration
    let client = reqwest::Client::builder()
        .unix_socket("/tmp/firecracker.socket")
        .build()
        .unwrap();

    let _ = client.put("http://localhost/boot-source")
        .json(&serde_json::json!({ "kernel_image_path": "vmlinux", "boot_args": "console=ttyS0 reboot=k panic=1 root=/dev/vda" })).send().await;

    let _ = client.put("http://localhost/drives/rootfs")
        .json(&serde_json::json!({ "drive_id": "rootfs", "path_on_host": "rootfs.ext4", "is_root_device": true, "is_read_only": false })).send().await;

    let _ = client.put("http://localhost/network-interfaces/eth0")
        .json(&serde_json::json!({ "iface_id": "eth0", "guest_mac": "AA:FC:00:00:00:01", "host_dev_name": "tap0" })).send().await;

    let _ = client
        .put("http://localhost/machine-config")
        .json(&serde_json::json!({ "vcpu_count": 1, "mem_size_mib": 128 }))
        .send()
        .await;

    let _ = client
        .put("http://localhost/actions")
        .json(&serde_json::json!({ "action_type": "InstanceStart" }))
        .send()
        .await;

    // The Custom Web Multiplexer
    let mut buf = [0u8; 1024];

    loop {
        tokio::select! {
            // Shovel Browser -> Firecracker
            msg = socket.recv() => {
                if let Some(Ok(Message::Text(text))) = msg {
                    if child_stdin.write_all(text.as_bytes()).await.is_err() { break; }
                } else {
                    println!("Browser closed");
                    break; } // Exit if browser closes
            }

            // Shovel Firecracker -> Browser
            bytes_read = child_stdout.read(&mut buf) => {
                if let Ok(n) = bytes_read {
                    if n == 0 { break; } // Exit if VM shuts down
                    let text = String::from_utf8_lossy(&buf[..n]).to_string();
                    if socket.send(Message::Text(text.into())).await.is_err() { break; }
                } else {
                    println!("VM closed");
                    break; }
            }
        }
    }

    println!("Session ended. Killing MicroVM...");
    let _ = child.kill().await;
}
