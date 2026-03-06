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

Phase 3: Configuring the MicroVM (The API)
Firecracker is headless. It requires configuration via a REST API over a Unix Domain Socket before it can boot.

1. Set the Boot Source (Kernel)
Tells the VMM where the kernel is and defines the boot arguments (mapping the console to ttyS0).

Bash
curl --unix-socket /tmp/firecracker.socket -X PUT \
  'http://localhost/boot-source' \
  -d '{
        "kernel_image_path": "vmlinux",
        "boot_args": "console=ttyS0 reboot=k panic=1"
    }'
2. Attach the Block Device (Hard Drive)
Maps the .ext4 file to the VM as the root partition.

Bash
curl --unix-socket /tmp/firecracker.socket -X PUT \
  'http://localhost/drives/rootfs' \
  -d '{
        "drive_id": "rootfs",
        "path_on_host": "rootfs.ext4",
        "is_root_device": true,
        "is_read_only": false
    }'
3. Set Machine Configuration
Defines the hardware constraints for the sandbox.

Bash
curl --unix-socket /tmp/firecracker.socket -X PUT \
  'http://localhost/machine-config' \
  -d '{
        "vcpu_count": 1,
        "mem_size_mib": 128
    }'
4. The Ignition
Powers on the virtual machine.

Bash
curl --unix-socket /tmp/firecracker.socket -X PUT \
  'http://localhost/actions' \
  -d '{ "action_type": "InstanceStart" }'
Phase 4: Execution Workflow
To run the entire stack from scratch:

Open Terminal A (Inside the fc-proxy directory).

Run cargo run.

The Rust program will spawn Firecracker in the background and sit quietly, waiting for output.

Open Terminal B (The Control Center).

Execute the first 3 curl commands to configure the VM.

Execute the 4th curl command (InstanceStart).

Return to Terminal A.

The Alpine Linux boot sequence will instantly stream to the screen.

Log in using root / root.

To shut down gracefully and exit the Rust proxy, type reboot inside the Alpine shell.

This is a fantastic foundation. Whenever you are ready to pick this up again, the next logical milestone is writing Rust code to send those HTTP API requests automatically, completely eliminating the need for Terminal B!
