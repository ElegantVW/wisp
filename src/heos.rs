//! Talk to speakers through `siren heos` — one protocol stack.

use crate::paths;
use serde_json::Value;
use std::process::Command;

pub fn siren_bin() -> Option<std::path::PathBuf> {
    paths::siren_candidates()
        .into_iter()
        .find(|p| !p.as_os_str().is_empty() && p.is_file())
}

pub fn cmd(args: &[&str]) -> Result<Value, String> {
    let bin = siren_bin().ok_or_else(|| {
        "siren engine missing — ./build.sh install --house in ~/siren".to_string()
    })?;
    let mut c = Command::new(&bin);
    c.arg("heos");
    for a in args {
        c.arg(a);
    }
    if !args.iter().any(|a| *a == "--json") {
        c.arg("--json");
    }
    let out = c.output().map_err(|e| format!("siren heos: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    if stdout.is_empty() {
        return Err(if stderr.is_empty() {
            format!("siren heos exited {}", out.status)
        } else {
            stderr
        });
    }
    serde_json::from_str(&stdout).map_err(|e| format!("siren heos json: {e} ({stdout})"))
}

/// Pin Siren to the HEOS source (aux → analog, net → follow).
pub fn match_source(source: &str) -> Result<String, String> {
    let bin = siren_bin().ok_or_else(|| "siren engine missing".to_string())?;
    let out = Command::new(&bin)
        .args(["audio", "match", source])
        .output()
        .map_err(|e| format!("siren audio match: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    if out.status.success() {
        Ok(stdout)
    } else {
        Err(if stderr.is_empty() { stdout } else { stderr })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn heos_cmd_goes_through_siren() {
        let src = std::fs::read_to_string("src/heos.rs").expect("heos.rs");
        let prod = src.split("mod tests").next().unwrap();
        assert!(prod.contains("arg(\"heos\")"));
        assert!(prod.contains("--json"));
        assert!(!prod.contains("sign_in"));
        assert!(
            !prod.contains("TcpStream"),
            "wisp does not open the speaker door itself"
        );
        assert!(
            prod.contains("audio\", \"match\""),
            "Wisp matches Siren after source"
        );
        let tui = std::fs::read_to_string("src/tui.rs").expect("tui.rs");
        let main = std::fs::read_to_string("src/main.rs").expect("main.rs");
        assert!(
            tui.contains(r#"["list", "--live"]"#),
            "TUI roster is live :1255, not a dead cache"
        );
        assert!(
            main.contains(r#"["list", "--live"]"#),
            "wisp discover/status is live :1255"
        );
    }
}
