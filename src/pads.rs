//! Pads via ether bt. Ether owns the motherboard Bluetooth;
//! Wisp only shells `ether bt …`. Never the BlueZ CLI, never D-Bus.

use serde_json::Value;
use std::process::Command;

fn ether_bin() -> String {
    crate::ap::ether_bin()
}

fn ether_bt(args: &[&str]) -> Result<String, String> {
    let mut cmd = Command::new(ether_bin());
    cmd.arg("bt");
    cmd.args(args);
    let out = cmd
        .output()
        .map_err(|e| format!("ether bt {}: {e}", args.first().copied().unwrap_or("")))?;
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    if out.status.success() {
        Ok(stdout)
    } else if !stderr.is_empty() {
        Err(stderr)
    } else if !stdout.is_empty() {
        Err(stdout)
    } else {
        Err("ether bt failed".into())
    }
}

pub fn status_line() -> String {
    ether_bt(&["status"]).unwrap_or_else(|e| e)
}

pub fn status() -> Result<Value, String> {
    let s = ether_bt(&["status"])?;
    serde_json::from_str(&s).map_err(|e| format!("ether bt status: {e}"))
}

pub fn scan() -> Result<Value, String> {
    let s = ether_bt(&["scan"])?;
    serde_json::from_str(&s).map_err(|e| format!("ether bt scan: {e}"))
}

pub fn pair(addr: &str) -> Result<String, String> {
    ether_bt(&["pair", addr])
}

pub fn connect(addr: &str) -> Result<String, String> {
    ether_bt(&["connect", addr])
}

pub fn disconnect(addr: &str) -> Result<String, String> {
    ether_bt(&["disconnect", addr])
}

pub fn forget(addr: &str) -> Result<String, String> {
    ether_bt(&["forget", addr])
}

pub fn pads_from(v: &Value) -> Vec<Value> {
    v.get("pads")
        .and_then(|p| p.as_array())
        .cloned()
        .unwrap_or_default()
}

pub fn trunc_addr(addr: &str) -> String {
    let p: Vec<&str> = addr.split(':').collect();
    if p.len() == 6 {
        format!("…:{}:{}", p[4], p[5])
    } else {
        "…".into()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn pads_transport_is_ether_bt_not_bluetoothctl() {
        let src = std::fs::read_to_string("src/pads.rs").expect("pads.rs");
        let prod = src.split("mod tests").next().unwrap();
        assert!(
            !prod.contains("bluetoothctl"),
            "wisp must not shell bluetoothctl"
        );
        assert!(!prod.contains("nmcli"));
        assert!(
            prod.contains(r#"cmd.arg("bt")"#),
            "wisp must route through ether bt"
        );
    }

    #[test]
    fn trunc_addr_hides_the_oui() {
        assert_eq!(super::trunc_addr("40:ec:99:c1:5e:07"), "…:5e:07");
    }
}
