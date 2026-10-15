use microvmctl::cli::run;
use microvmctl::*;

fn cli(a: &[&str]) -> (i32, String) {
    let args: Vec<String> = a.iter().map(|s| s.to_string()).collect();
    let mut out = Vec::new();
    let rc = run(&args, &mut out);
    (rc, String::from_utf8(out).unwrap())
}

#[test]
fn defaults_are_confined() {
    let c = Config::new("k", "r", "uname -r");
    assert_eq!(c.validate(), Ok(()));
    assert!(c.rootfs_read_only);
    assert!(c.tap.is_none());
    assert!(c.boot_args().contains("-c \"uname -r; reboot -f\""));
}

#[test]
fn every_widening_is_refused() {
    let base = Config::new("k", "r", "x");
    let mut c = base.clone();
    c.rootfs_read_only = false;
    assert_eq!(c.validate(), Err(Refusal::WritableRoot));
    let mut c = base.clone();
    c.vcpus = 9;
    assert_eq!(c.validate(), Err(Refusal::Vcpus(9)));
    let mut c = base.clone();
    c.mem_mib = 64;
    assert_eq!(c.validate(), Err(Refusal::Memory(64)));
    let mut c = base;
    c.tap = Some("tap0".into());
    assert_eq!(c.validate(), Err(Refusal::Network));
}

#[test]
fn json_has_the_three_sections() {
    let j = Config::new("/k", "/r", "id").to_json();
    for part in [
        "boot-source",
        "drives",
        "machine-config",
        "\"is_read_only\":true",
        "vcpu_count\":1",
    ] {
        assert!(j.contains(part), "{part}");
    }
}

#[test]
fn cli_paths() {
    assert!(cli(&["config", "/k", "/r", "id"])
        .1
        .contains("\"kernel_image_path\":\"/k\""));
    assert_eq!(cli(&["validate", "1", "256", "ro"]), (0, "OK\n".into()));
    assert_eq!(
        cli(&["validate", "1", "256", "rw"]),
        (1, "REFUSED WritableRoot\n".into())
    );
    assert_eq!(cli(&["validate", "x", "256", "ro"]).0, 2);
    assert!(cli(&["args", "id"]).1.contains("init=/bin/sh"));
    assert_eq!(
        cli(&["contract"]),
        (
            0,
            "contract: microvm-default-config-is-confined OK\n".into()
        )
    );
    assert_eq!(cli(&[]).0, 2);
    for c in ["config", "validate", "args", "bogus"] {
        assert_eq!(cli(&[c]).0, 2);
    }
}

#[test]
fn binary_runs_end_to_end() {
    let o = std::process::Command::new(env!("CARGO_BIN_EXE_microvmctl"))
        .arg("contract")
        .output()
        .unwrap();
    assert!(o.status.success());
    assert!(String::from_utf8_lossy(&o.stdout)
        .contains("contract: microvm-default-config-is-confined OK"));
}
