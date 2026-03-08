# Firecracker Web Sandbox

A lightweight MicroVM sandbox orchestrator built in Rust. It leverages AWS Firecracker and KVM to securely boot Alpine Linux instances in milliseconds, streaming an interactive, highly responsive, and dynamically resizable console directly to the browser via WebSockets and VSOCK. 

Mimics the core architectures of interactive coding platforms like SadServers, repl.it, and AWS Fargate.

## Features
- **Lightning Fast Boot**: Boots hardware-isolated Alpine Linux MicroVMs in sub-second times.
- **Browser-based Terminal**: Streams the MicroVM's serial console (`/dev/ttyS0`) to a web browser using WebSockets and `xterm.js`.
- **Dynamic Terminal Resizing**: Implements a custom `guest-agent` to route WebSocket resize payloads through a VSOCK channel, issuing `ioctl` commands to automatically redraw terminal apps like `top` and `vim`.
- **Pre-configured Networking**: Demonstrates creating custom TAP devices and configuring IP forwarding/masquerading to provide the MicroVMs with instant internet access.

## Architecture
1. **Host Orchestrator (`fc-proxy`)**: A Rust/Axum web server running on the host that spawns Firecracker processes, manages Unix Domain Sockets, configures the MicroVM APIs, and acts as a WebSocket relay bridge.
2. **MicroVM Engine (`firecracker`)**: The AWS open-source VMM that interacts with KVM to provide secure, lightweight hardware virtualization.
3. **Guest OS (`Alpine Linux`)**: A hyper-minimal Linux distribution running a stripped-down v6.1 kernel.
4. **Guest Agent (`guest-agent`)**: A background service inside the MicroVM compiled with `musl` that listens on a VSOCK port for resize events to dynamically update the internal TTY dimensions.

---

## 🚀 Getting Started

To run this project, you must be on a **Linux** machine with KVM enabled. 

### 1. Download Assets
Since the `firecracker` VMM binary, the `vmlinux` kernel, and the `rootfs.ext4` filesystem are large binaries, they are excluded from this repository. 

Run the included download script to fetch the AWS-provided minimal Alpine test images:
```bash
./download_kernel_and_fs.sh
```

### 2. Configure Host Networking
You need to create a TAP device and configure your host machine (e.g., Ubuntu/Mint) to route the VM's traffic using NAT. 

Run the following commands to configure the `tap0` interface:
```bash
sudo ip tuntap add tap0 mode tap
sudo ip addr add 172.16.0.1/24 dev tap0
sudo ip link set tap0 up
sudo sh -c "echo 1 > /proc/sys/net/ipv4/ip_forward"

# Note: change $HOST_IFACE below to your primary internet interface (e.g., eth0, wlan0)
HOST_IFACE=$(ip route show default | awk '/default/ {print $5}')
sudo iptables -t nat -A POSTROUTING -o $HOST_IFACE -j MASQUERADE
sudo iptables -A FORWARD -i tap0 -o $HOST_IFACE -j ACCEPT
sudo iptables -A FORWARD -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT
```

### 3. Run the Orchestrator
Start the Rust web server:
```bash
cd fc-proxy
cargo run
```

### 4. Connect
Open your browser and navigate to the orchestrator. If running locally:
```
http://localhost:3000
```
If you deployed this to a remote cloud server, use your server's public IP address:
```
http://[YOUR_SERVER_IP]:3000
```
Login with the username `root` and password `root`. 
You can type `ping google.com` to verify internet connectivity, or run `top` and resize your browser window to watch the terminal dynamically adapt!

---

## 🛠 Building the Custom Guest Agent
If you want to modify how the VM handles terminal resizing, you can rebuild the guest agent. Because Alpine Linux uses `musl` libc instead of standard GNU `glibc`, the agent must be cross-compiled:

```bash
cd guest-agent
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
```
*(You will need to mount the `rootfs.ext4` file and replace the agent binary inside it to apply updates).*
