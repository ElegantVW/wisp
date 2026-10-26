# Wisp

Rust remote for house lanterns and HEOS. No Denon account. No firmware flash.

- Engine path: `~/.local/lib/faeos/wisp`. Launcher `~/bin/wisp` is a script.
- HEOS transport is `siren heos --json`. Do not add a second CLI stack.
- House AP transport is `ether ap on|off|status` (iw + hostapd). Wisp never calls nmcli or hostapd itself.
- `~/.config/ether/ap.env` is 0600 local (ether falls back to `~/.config/wisp/ap.env`). Never git, never logs, never memory.
- Tests must not raise Cave/Office volume.
- Tapo color engine waits on the laptop tree.
