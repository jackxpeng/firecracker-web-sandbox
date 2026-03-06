use std::process::Stdio;
use tokio::io::{self};
use tokio::process::Command;

#[tokio::main]
async fn main() -> io::Result<()> {
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

    // 3. Extract the handles to the "wire"
    let mut child_stdin = child.stdin.take().expect("Failed to capture stdin");
    let mut child_stdout = child.stdout.take().expect("Failed to capture stdout");

    let mut host_stdin = io::stdin();
    let mut host_stdout = io::stdout();

    println!("--- FIRECRACKER RUNNING IN THE BACKGROUND ---");
    println!("Go to Terminal B and run your 4 curl commands to boot the VM!");

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
