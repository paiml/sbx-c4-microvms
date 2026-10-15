//! A Firecracker microVM configuration builder with the checks the course
//! relies on: read-only root file system, bounded resources, no network by default.

pub mod cli;

/// What a microVM is allowed to be.
#[derive(Debug, PartialEq, Clone)]
pub struct Config {
    pub kernel: String,
    pub rootfs: String,
    pub vcpus: u32,
    pub mem_mib: u32,
    pub command: String,
    pub rootfs_read_only: bool,
    pub tap: Option<String>,
}

/// Reasons a configuration is refused.
#[derive(Debug, PartialEq)]
pub enum Refusal {
    WritableRoot,
    Vcpus(u32),
    Memory(u32),
    Network,
}

impl Config {
    pub fn new(kernel: &str, rootfs: &str, command: &str) -> Config {
        Config {
            kernel: kernel.into(),
            rootfs: rootfs.into(),
            vcpus: 1,
            mem_mib: 256,
            command: command.into(),
            rootfs_read_only: true,
            tap: None,
        }
    }

    /// First rule broken, if any.
    pub fn validate(&self) -> Result<(), Refusal> {
        if !self.rootfs_read_only {
            return Err(Refusal::WritableRoot);
        }
        if !(1..=4).contains(&self.vcpus) {
            return Err(Refusal::Vcpus(self.vcpus));
        }
        if !(128..=1024).contains(&self.mem_mib) {
            return Err(Refusal::Memory(self.mem_mib));
        }
        if self.tap.is_some() {
            return Err(Refusal::Network);
        }
        Ok(())
    }

    /// Kernel command line: serial console, no PCI, run one command then reboot.
    pub fn boot_args(&self) -> String {
        format!(
            "console=ttyS0 reboot=k panic=1 pci=off root=/dev/vda ro init=/bin/sh -- -c \"{}; reboot -f\"",
            self.command
        )
    }

    /// The Firecracker `--config-file` JSON.
    pub fn to_json(&self) -> String {
        format!(
            "{{\"boot-source\":{{\"kernel_image_path\":\"{}\",\"boot_args\":{:?}}},\"drives\":[{{\"drive_id\":\"rootfs\",\"path_on_host\":\"{}\",\"is_root_device\":true,\"is_read_only\":{}}}],\"machine-config\":{{\"vcpu_count\":{},\"mem_size_mib\":{}}}}}",
            self.kernel,
            self.boot_args(),
            self.rootfs,
            self.rootfs_read_only,
            self.vcpus,
            self.mem_mib
        )
    }
}
