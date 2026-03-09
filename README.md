# Firecracker Web Sandbox

A lightweight MicroVM sandbox orchestrator built in Rust. It leverages AWS Firecracker and KVM to securely boot Alpine Linux instances in milliseconds, streaming an interactive, highly responsive, and dynamically resizable console directly to the browser via WebSockets and VSOCK.

Mimics the core architectures of interactive coding platforms like SadServers, repl.it, and AWS Fargate.

## 🌟 Features
- **Lightning Fast Boot**: Boots hardware-isolated Alpine Linux MicroVMs in sub-second times.
- **Multi-Tenant Isolation**: Dynamically generates unique `/30` subnets (`10.200.x.x`) and persistent TAP interfaces for every concurrent user.
- **Browser-based Terminal**: Streams the MicroVM's serial console (`/dev/ttyS0`) to a web browser using WebSockets and `xterm.js`.
- **Dynamic Terminal Resizing**: Implements a custom `guest-agent` via VSOCK to issue `ioctl` commands, automatically redrawing apps like `top` and `vim`.
- **Automated Networking**: Uses kernel boot arguments to auto-configure Guest IP addresses, providing instant internet access without manual setup.

## 🏗 Architecture
1. **Host Orchestrator (`fc-proxy`)**: A Rust/Axum web server that dynamically manages TAP devices, calculates unique IP blocks, and acts as a WebSocket-to-Serial bridge.
2. **MicroVM Engine (`firecracker`)**: The AWS VMM interacting with KVM to provide secure, lightweight hardware virtualization.
3. **Guest OS (`Alpine Linux`)**: A hyper-minimal Linux distribution running a stripped-down v6.1 kernel.
4. **Guest Agent (`guest-agent`)**: A background service inside the MicroVM that listens on a VSOCK port for resize events.
5. **Init System (`OpenRC`)**: A custom startup script (`fc-network`) that parses `/proc/cmdline` to auto-configure the network on boot.

---

## 🚀 Getting Started

### 1. Download Assets
Fetch the AWS-provided minimal Alpine test images and kernel:
```bash
./download_kernel_and_fs.sh
```

### 2. Configure Persistent Host NAT (One-Time Setup)
To allow your dynamic MicroVMs to reach the internet, you must enable IP forwarding and create a "Catch-All" NAT rule on your host machine (laptop or DigitalOcean Droplet).
```bash
# Enable IP Forwarding
sudo sysctl -w net.ipv4.ip_forward=1

# Configure NAT for the 10.200.0.0/16 MicroVM range
HOST_IFACE=$(ip route show default | awk '/default/ {print $5}')
sudo iptables -t nat -A POSTROUTING -s 10.200.0.0/16 -o $HOST_IFACE -j MASQUERADE
sudo iptables -A FORWARD -s 10.200.0.0/16 -j ACCEPT
sudo iptables -A FORWARD -d 10.200.0.0/16 -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT
```

### 3. Build and Run the Orchestrator
The orchestrator requires privileges to manage network interfaces. You can either run as root or grant the binary specific Linux capabilities.

**Option A: The Capability Method (Recommended)**
```bash
cd fc-proxy
cargo build
sudo setcap cap_net_admin+ep target/debug/fc-proxy
./target/debug/fc-proxy
```

**Option B: The Simple Method**
```bash
cd fc-proxy
cargo build
sudo ./target/debug/fc-proxy
```

### 4. Connect
Navigate to `http://localhost:3000` (or your Server IP).
The MicroVM will boot and automatically configure itself with a unique IP (e.g., `10.200.1.2`).
Try `ping google.com` immediately—it works out of the box!

---

## 🛠 Advanced: The Guest Auto-Config Script
The Guest OS automatically configures its network by reading the following custom boot arguments passed by the Rust orchestrator:
`fc_ip=10.200.x.2 fc_gw=10.200.x.1`

The internal script at `/etc/init.d/fc-network` handles this logic:
```bash
# Extracts IPs from /proc/cmdline and applies them to eth0
FC_IP=$(cat /proc/cmdline | sed 's/.*fc_ip=\([^ ]*\).*/\1/')
ip addr add "$FC_IP/30" dev eth0
ip link set eth0 up
ip route add default via "$FC_GW"
```
