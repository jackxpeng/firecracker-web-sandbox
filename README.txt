roject: MicroVM Orchestrator with Firecracker & Rust
Objective: Build a custom "backend" that downloads, configures, and boots a hardware-isolated Linux MicroVM, and routes its interactive serial console through a custom Rust proxy. This mimics the core architecture of platforms like AWS Fargate or interactive coding sandboxes.

Phase 1: Acquiring the Assets
To boot a MicroVM, three specific binary components are required: the Virtual Machine Monitor (VMM), a Linux kernel, and a Root Filesystem (the virtual hard drive).

1. The Firecracker Binary (v1.14.2)
The VMM engine that handles hardware emulation (KVM) and API requests.

Bash
# Download and extract the x86_64 binary bundle
curl -L https://github.com/firecracker-microvm/firecracker/releases/download/v1.14.2/firecracker-v1.14.2-x86_64.tgz -o firecracker.tgz
tar -xvf firecracker.tgz
mv release-v1.14.2-x86_64/firecracker-v1.14.2-x86_64 firecracker
chmod +x firecracker
2. The "Hello World" Kernel & RootFS
Minimalist test assets provided by the Firecracker team. The OS is Alpine Linux, and the kernel is a stripped-down 4.14 build.

Bash
# Download the kernel
curl -L https://s3.amazonaws.com/spec.ccfc.min/img/hello/kernel/hello-vmlinux.bin -o vmlinux

# Download the Alpine filesystem
curl -L https://s3.amazonaws.com/spec.ccfc.min/img/hello/fsfiles/hello-rootfs.ext4 -o rootfs.ext4
Phase 2: The Rust Orchestrator (fc-proxy)
Instead of interacting with Firecracker directly, we wrote a Rust program to act as an orchestrator. It spawns Firecracker as a background child process and asynchronously "shovels" bytes between the host terminal and the guest VM's serial port.

Project Setup:

Bash
cargo new fc-proxy
cd fc-proxy
cargo add tokio --features full
cargo add crossterm
The Code (src/main.rs):
    This code handles process creation, pipe capture, terminal raw mode (to prevent double-echoing and interpret special keystrokes correctly), and multiplexed I/O.

