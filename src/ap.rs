//! House AP via ether. Ether owns the radio path (nl80211 ELF, no
//! NetworkManager); wisp only shells `ether ap …` and never reads ap.env
//! itself. Credentials live in ~/.config/ether/ap.env (ether falls back
//! to ~/.config/wisp/ap.env) and are never printed.

use std::process::Command;

pub(crate) fn ether_bin() -> String {
    if let Some(home) = std::env::var_os("HOME") {
        let h = std::path::Path::new(&home);
        for p in [
            h.join(".local/lib/faeos/ether"),
            h.join("bin").join("ether"),
        ] {
            if p.is_file() {
                return p.to_string_lossy().into_owned();
            }
        }
    }
    "/usr/bin/ether".into()
}

fn ether_ap(args: &str) -> Result<String, String> {
    let out = Command::new(ether_bin())
        .args(["ap", args])
        .output()
        .map_err(|e| format!("ether ap {args}: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

pub fn raise() -> Result<String, String> {
    ether_ap("on")
}

pub fn lower() -> Result<String, String> {
    ether_ap("off")
}

pub fn status_line() -> String {
    ether_ap("status").unwrap_or_else(|e| e)
}

#[cfg(test)]
mod tests {
    use super::ether_bin;

    #[test]
    fn ap_transport_is_ether_not_nmcli() {
        let src = std::fs::read_to_string("src/ap.rs").expect("ap.rs");
        let prod = src.split("mod tests").next().unwrap();
        assert!(
            !prod.contains("nmcli"),
            "wisp must not shell nmcli for the AP"
        );
        for needle in [
            "ether_ap(\"on\")",
            "ether_ap(\"off\")",
            "ether_ap(\"status\")",
        ] {
            assert!(prod.contains(needle), "missing `{needle}`");
        }
        assert!(
            prod.contains(r#"["ap", args]"#),
            "wisp must route through `ether ap …`"
        );
    }

    #[test]
    fn ether_bin_prefers_engine() {
        let b = ether_bin();
        let home = std::env::var_os("HOME").expect("HOME");
        let engine = std::path::Path::new(&home).join(".local/lib/faeos/ether");
        let face = std::path::Path::new(&home).join("bin").join("ether");
        if engine.is_file() {
            assert_eq!(b, engine.to_string_lossy());
        } else if face.is_file() {
            assert_eq!(b, face.to_string_lossy());
        } else {
            assert_eq!(b, "/usr/bin/ether");
        }
    }
}
