# Wisp — the lanterns 🎐

```
  ^
 / \
| * |
 \_/
```

House lanterns and the HEOS remote. Pick a speaker, a source, the house radio — no Denon app, no vendor cloud. Rust TUI.

For: HEOS on the LAN, house AP/enlist, gamepads through Ether. Not for: the Denon phone app, NetworkManager, bluetoothctl.

```
git clone git@github.com:ElegantVW/wisp.git ~/wisp
cd ~/wisp && ./build.sh install --house
wisp
```

One-shots: `wisp status` · `wisp discover` · `wisp ap on|off|status` · `wisp pads`.

Speakers are driven through `siren heos` (stock CLI on the LAN). Wisp does not open that door itself.

## Faces

Tab / Shift-Tab cycle faces using Grove's shared `cycle_next` /
`cycle_previous` bindings.

- **speakers** — roster, vol, mute, play/stop, group
- **source** — `a` AUX · `b` Bluetooth · `n` network (Siren follow/radio)
- **enlist** — house AP + WPS, or one RJ45 singleton
- **ap** — house hotspot via `ether ap`
- **pads** — gamepads via `ether bt`. Wisp never opens HCI or bluetoothctl.

One ethernet jack is a singleton. A fleet groups on the house AP.

## ap.env

Ether owns the AP credentials (`~/.config/ether/ap.env`). The PSK is never printed — status only says `psk set`. Never commit that file. Wisp shells `ether ap …` (same shape as `siren heos`).

## License

MIT — see [LICENSE](LICENSE).
