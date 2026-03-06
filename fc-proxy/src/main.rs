use std::process::Stdio;
use std::time::Duration;
use tokio::io::{self};
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Clean up any old socket
    let _ = std::fs::remove_file("/tmp/firecracker.socket");

    println!("Spawning Firecracker engine...");

    // 2. Start Firecracker as a child process and capture its pipes
    // (Assuming the firecracker binary is one directory up from your fc-proxy folder)
    let mut child = Command::new("./firecracker")
        .current_dir("..")
        .arg("--api-sock")
        .arg("/tmp/firecracker.socket")
        .arg("--enable-pci")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    //

    // 3. Extract the handles to the "wire"
    let mut child_stdin = child.stdin.take().expect("Failed to capture stdin");
    let mut child_stdout = child.stdout.take().expect("Failed to capture stdout");

    let mut host_stdin = io::stdin();
    let mut host_stdout = io::stdout();

    // Give Firecracker a moment to create the socket file
    tokio::time::sleep(Duration::from_millis(500)).await;

    println!("confiruing MicroVM via API...");

    let client = reqwest::Client::builder()
        .unix_socket("/tmp/firecracker.socket")
        .build()?;

    // 1. Set up Boot Source
    client
        .put("http://localhost/boot-source")
        .json(&serde_json::json!({
            "kernel_image_path": "vmlinux",
            "boot_args": "console=ttyS0 reboot=k panic=1"
        }))
        .send()
        .await?;

    // 2. Attach the Block Device (Hard Drive)
    client
        .put("http://localhost/drives/rootfs")
        .json(&serde_json::json!({
            "drive_id": "rootfs",
            "path_on_host": "rootfs.ext4",
            "is_root_device": true,
            "is_read_only": false
        }))
        .send()
        .await?;

    // 3. Set Machine Configuration
    client
        .put("http://localhost/machine-config")
        .json(&serde_json::json!({
            "vcpu_count": 1,
            "mem_size_mib": 128
        }))
        .send()
        .await?;

    println!("Ignition! Starting VM...");

    // 4. Start the VM
    client
        .put("http://localhost/actions")
        .json(&serde_json::json!({"action_type": "InstanceStart"}))
        .send()
        .await?;

    // 4. The Multiplexer (The Byte Shoveler)
    tokio::select! {
        // Shovel Host Keyboard -> Firecracker Stdin
        res = io::copy(&mut host_stdin, &mut child_stdin) => {
            println!("Keyboard stream ended: {:?}", res);
        }
        // Shovel Firecracker Stdout -> Host Screen
        res = io::copy(&mut child_stdout, &mut host_stdout) => {
            println!("VM stream ended: {:?}", res);
        }
        // Wait in case the Firecracker process crashes or exits
        _ = child.wait() => {
            println!("Firecracker process exited normally.");
        }
    }

    Ok(())
}
