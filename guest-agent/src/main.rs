use libc::{ioctl, winsize, TIOCSWINSZ};
use serde::Deserialize;
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;
use vsock::{VsockListener, VMADDR_CID_ANY};

#[derive(Deserialize)]
struct ResizeMsg {
    cols: u16,
    rows: u16,
}

fn main() {
    // 1. Open the Terminal Device
    // We open the specific serial port Firecracker uses for the console.
    let tty = OpenOptions::new()
        .write(true)
        .open("/dev/ttyS0")
        .expect("Failed to open /dev/ttyS0");
    let fd = tty.as_raw_fd();

    // 2. Open the Vsock Listener
    // CID_ANY means "listen for anyone trying to talk to me".
    // Port 5000 is an arbitrary port we will configure Firecracker to route to.
    let listener = VsockListener::bind_with_cid_port(VMADDR_CID_ANY, 5000)
        .expect("Failed to bind vsock listener");

    println!("Guest agent listening on Vsock port 5000...");

    // 3. The Event Loop
    for stream in listener.incoming() {
        match stream {
            Ok(mut s) => {
                let mut buf = [0; 1024];
                if let Ok(n) = std::io::Read::read(&mut s, &mut buf) {
                    if n > 0 {
                        // Parse the JSON coming from your web browser
                        let msg_str = String::from_utf8_lossy(&buf[..n]);
                        if let Ok(msg) = serde_json::from_str::<ResizeMsg>(&msg_str) {
                            // 4. The Magic System Call
                            // Construct the C-style struct the kernel expects
                            let ws = winsize {
                                ws_row: msg.rows,
                                ws_col: msg.cols,
                                ws_xpixel: 0,
                                ws_ypixel: 0,
                            };

                            // Unsafe because we are making a raw C FFI call to the kernel
                            unsafe {
                                ioctl(fd, TIOCSWINSZ, &ws);
                            }
                        }
                    }
                }
            }
            Err(e) => eprintln!("Vsock connection failed: {}", e),
        }
    }
}
