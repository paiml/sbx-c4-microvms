//! Command line behind the demos. `run` returns the exit code.

use crate::{Config, Refusal};
use std::io::Write;

type Handler = fn(&[&str], &mut dyn Write) -> i32;

const COMMANDS: [(&str, Handler); 4] = [
    ("config", config),
    ("validate", validate),
    ("args", args_cmd),
    ("contract", contract),
];

pub fn run(args: &[String], out: &mut dyn Write) -> i32 {
    let a: Vec<&str> = args.iter().map(String::as_str).collect();
    let handler = COMMANDS.iter().find(|(name, _)| a.first() == Some(name));
    match handler {
        Some((_, h)) => h(&a[1..], out),
        None => usage(out),
    }
}

fn usage(out: &mut dyn Write) -> i32 {
    let _ = writeln!(out, "usage: microvmctl config|validate|args|contract");
    2
}

/// `config <kernel> <rootfs> <command>`: print the Firecracker JSON.
fn config(args: &[&str], out: &mut dyn Write) -> i32 {
    let [kernel, rootfs, command] = args else {
        return usage(out);
    };
    let _ = writeln!(out, "{}", Config::new(kernel, rootfs, command).to_json());
    0
}

/// `validate <vcpus> <mem_mib> <ro|rw>`: check a configuration.
fn validate(args: &[&str], out: &mut dyn Write) -> i32 {
    let [vcpus, mem, mode] = args else {
        return usage(out);
    };
    let (Ok(v), Ok(m)) = (vcpus.parse::<u32>(), mem.parse::<u32>()) else {
        return usage(out);
    };
    let mut c = Config::new("k", "r", "true");
    c.vcpus = v;
    c.mem_mib = m;
    c.rootfs_read_only = *mode == "ro";
    match c.validate() {
        Ok(()) => {
            let _ = writeln!(out, "OK");
            0
        }
        Err(r) => {
            let _ = writeln!(out, "REFUSED {r:?}");
            1
        }
    }
}

fn args_cmd(args: &[&str], out: &mut dyn Write) -> i32 {
    let [command] = args else { return usage(out) };
    let _ = writeln!(out, "{}", Config::new("k", "r", command).boot_args());
    0
}

/// Runtime-enforced contract: the default config is acceptable, and every
/// widening is refused.
fn contract(_args: &[&str], out: &mut dyn Write) -> i32 {
    let ok = Config::new("k", "r", "true");
    assert_eq!(ok.validate(), Ok(()));
    let mut w = ok.clone();
    w.rootfs_read_only = false;
    assert_eq!(w.validate(), Err(Refusal::WritableRoot));
    let mut n = ok.clone();
    n.tap = Some("tap0".into());
    assert_eq!(n.validate(), Err(Refusal::Network));
    let _ = writeln!(out, "contract: microvm-default-config-is-confined OK");
    0
}
