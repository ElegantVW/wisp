# Wisp

House lanterns and the HEOS remote. No Denon app, no vendor cloud.

Rust. TUI: `wisp`. One-shots: `wisp status` · `wisp discover` · `wisp ap on|off|status`.

Speakers are driven through `siren heos` (stock CLI on the LAN). Wisp does not open that door itself.

## Faces

- **speakers** — roster, vol, mute, play/stop, group
- **source** — `a` AUX · `b` Bluetooth · `n` network (Siren follow/radio)
- **enlist** — house AP + WPS (Connect ~3s) or one RJ45 singleton
- **ap** — house hotspot via `ether ap` (iw + hostapd; NetworkManager-free). Wisp only shells `ether ap on|off|status` and shows ether's status line.

One ethernet jack is a singleton. A fleet groups on the house AP.

## ap.env

Ether owns the AP credentials. It reads `~/.config/ether/ap.env`
(`ETHER_AP_IFACE` / `ETHER_AP_SSID` / `ETHER_AP_PSK`, mode 600) and falls back
to wisp's older `~/.config/wisp/ap.env` (`WISP_AP_*`) so existing files keep
working. The PSK is never printed — status only says `psk set`.

Never commit either file. Wisp itself never reads ap.env; it shells
`ether ap …` (same shape as `siren heos`).

## Build

```bash
./build.sh install --house
```
