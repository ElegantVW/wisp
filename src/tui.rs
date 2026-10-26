//! Five faces: speakers · source · enlist · ap · pads.

use crate::{ap, heos, pads};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use serde_json::Value;
use std::io::{self, stdout};
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Face {
    Speakers,
    Source,
    Enlist,
    Ap,
    Pads,
}

const FACES: [Face; 5] = [
    Face::Speakers,
    Face::Source,
    Face::Enlist,
    Face::Ap,
    Face::Pads,
];

impl Face {
    fn title(self) -> &'static str {
        match self {
            Face::Speakers => "speakers",
            Face::Source => "source",
            Face::Enlist => "enlist",
            Face::Ap => "ap",
            Face::Pads => "pads",
        }
    }
}

struct App {
    face: usize,
    sel: usize,
    players: Vec<Value>,
    pads: Vec<Value>,
    pad_sel: usize,
    msg: String,
    ok: bool,
}

impl App {
    fn new() -> Self {
        let mut a = Self {
            face: 0,
            sel: 0,
            players: Vec::new(),
            pads: Vec::new(),
            pad_sel: 0,
            msg: String::new(),
            ok: true,
        };
        a.refresh();
        if a.players.is_empty() {
            a.face = 2; // enlist
        }
        a
    }

    fn face(&self) -> Face {
        FACES[self.face]
    }

    fn refresh(&mut self) {
        match heos::cmd(&["list", "--live"]) {
            Ok(v) => {
                self.players = v
                    .get("players")
                    .and_then(|p| p.as_array())
                    .cloned()
                    .unwrap_or_default();
                self.ok = v.get("ok").and_then(|x| x.as_bool()).unwrap_or(true);
                if self.players.is_empty() {
                    self.msg = "no speakers on the LAN — enlist (AP + WPS, or one RJ45)".into();
                } else {
                    self.msg = format!("{} boxes", self.players.len());
                }
            }
            Err(e) => {
                self.players.clear();
                self.ok = false;
                self.msg = e;
            }
        }
        if !self.players.is_empty() {
            self.sel = self.sel.min(self.players.len() - 1);
        } else {
            self.sel = 0;
        }
        if let Ok(v) = pads::status() {
            self.pads = pads::pads_from(&v);
        }
        if !self.pads.is_empty() {
            self.pad_sel = self.pad_sel.min(self.pads.len() - 1);
        } else {
            self.pad_sel = 0;
        }
    }

    fn selected_pad_addr(&self) -> Option<String> {
        self.pads
            .get(self.pad_sel)
            .and_then(|p| p.get("addr"))
            .and_then(|n| n.as_str())
            .map(|s| s.to_string())
    }

    fn selected_name(&self) -> Option<String> {
        self.players
            .get(self.sel)
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .map(|s| s.to_string())
    }

    fn run_heos(&mut self, args: &[&str]) {
        match heos::cmd(args) {
            Ok(v) => {
                self.ok = v.get("ok").and_then(|x| x.as_bool()).unwrap_or(true);
                self.msg = v
                    .get("msg")
                    .and_then(|m| m.as_str())
                    .or_else(|| v.get("error").and_then(|m| m.as_str()))
                    .unwrap_or("ok")
                    .to_string();
            }
            Err(e) => {
                self.ok = false;
                self.msg = e;
            }
        }
        self.refresh();
    }
}

pub fn run() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut term = Terminal::new(ratatui::backend::CrosstermBackend::new(stdout))?;
    let mut app = App::new();
    loop {
        term.draw(|f| draw(f, &app))?;
        if event::poll(Duration::from_millis(400))? {
            if let Event::Key(k) = event::read()? {
                if handle(&mut app, k.code, k.modifiers) {
                    break;
                }
            }
        }
    }
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

fn handle(app: &mut App, code: KeyCode, mods: KeyModifiers) -> bool {
    if crate::grove_keys::is_cycle_next(code, mods) {
        app.face = (app.face + 1) % FACES.len();
        return false;
    }
    if crate::grove_keys::is_cycle_previous(code, mods) {
        app.face = (app.face + FACES.len() - 1) % FACES.len();
        return false;
    }
    if app.face() == Face::Pads {
        return handle_pads(app, code, mods);
    }
    match code {
        KeyCode::Char('q') | KeyCode::Esc => return true,
        KeyCode::Char('r') => app.refresh(),
        KeyCode::Char('j') | KeyCode::Down => {
            if !app.players.is_empty() {
                app.sel = (app.sel + 1).min(app.players.len() - 1);
            }
        }
        KeyCode::Char('k') | KeyCode::Up => app.sel = app.sel.saturating_sub(1),
        KeyCode::Char('+') | KeyCode::Char('=') => {
            if let Some(n) = app.selected_name() {
                app.run_heos(&["vol", "+5", "--speaker", &n]);
            }
        }
        KeyCode::Char('-') | KeyCode::Char('_') => {
            if let Some(n) = app.selected_name() {
                app.run_heos(&["vol", "-5", "--speaker", &n]);
            }
        }
        KeyCode::Char('m') => {
            if let Some(n) = app.selected_name() {
                app.run_heos(&["mute", "toggle", "--speaker", &n]);
            }
        }
        KeyCode::Char('p') | KeyCode::Char(' ') => {
            if let Some(n) = app.selected_name() {
                app.run_heos(&["play", "--speaker", &n]);
            }
        }
        KeyCode::Char('s') => {
            if let Some(n) = app.selected_name() {
                app.run_heos(&["stop", "--speaker", &n]);
            }
        }
        KeyCode::Char('a') => source(app, "aux"),
        KeyCode::Char('b') => source(app, "bt"),
        KeyCode::Char('n') => source(app, "net"),
        KeyCode::Char('g') => {
            if let Some(n) = app.selected_name() {
                app.run_heos(&["group", "--speaker", &n]);
            }
        }
        KeyCode::Char('u') => {
            if let Some(n) = app.selected_name() {
                app.run_heos(&["ungroup", "--speaker", &n]);
            }
        }
        KeyCode::Char('A') if app.face() == Face::Ap => match ap::raise() {
            Ok(m) => {
                app.ok = true;
                app.msg = m;
            }
            Err(e) => {
                app.ok = false;
                app.msg = e;
            }
        },
        KeyCode::Char('Z') if app.face() == Face::Ap => match ap::lower() {
            Ok(m) => {
                app.ok = true;
                app.msg = m;
            }
            Err(e) => {
                app.ok = false;
                app.msg = e;
            }
        },
        _ => {}
    }
    false
}

fn handle_pads(app: &mut App, code: KeyCode, _mods: KeyModifiers) -> bool {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => return true,
        KeyCode::Char('r') => app.refresh(),
        KeyCode::Char('j') | KeyCode::Down => {
            if !app.pads.is_empty() {
                app.pad_sel = (app.pad_sel + 1).min(app.pads.len() - 1);
            }
        }
        KeyCode::Char('k') | KeyCode::Up => app.pad_sel = app.pad_sel.saturating_sub(1),
        KeyCode::Char('s') => match pads::scan() {
            Ok(v) => {
                app.ok = v.get("ok").and_then(|x| x.as_bool()).unwrap_or(true);
                app.pads = pads::pads_from(&v);
                app.pad_sel = 0;
                app.msg = format!(
                    "{} pad(s) — hold SHARE+PS on the Battletron",
                    app.pads.len()
                );
            }
            Err(e) => {
                app.ok = false;
                app.msg = e;
            }
        },
        KeyCode::Enter => {
            if let Some(a) = app.selected_pad_addr() {
                match pads::pair(&a) {
                    Ok(m) => {
                        app.ok = true;
                        app.msg = m;
                    }
                    Err(e) => {
                        app.ok = false;
                        app.msg = e;
                    }
                }
                app.refresh();
            }
        }
        KeyCode::Char('c') => {
            if let Some(a) = app.selected_pad_addr() {
                match pads::connect(&a) {
                    Ok(m) => {
                        app.ok = true;
                        app.msg = m;
                    }
                    Err(e) => {
                        app.ok = false;
                        app.msg = e;
                    }
                }
                app.refresh();
            }
        }
        KeyCode::Char('d') => {
            if let Some(a) = app.selected_pad_addr() {
                match pads::disconnect(&a) {
                    Ok(m) => {
                        app.ok = true;
                        app.msg = m;
                    }
                    Err(e) => {
                        app.ok = false;
                        app.msg = e;
                    }
                }
                app.refresh();
            }
        }
        KeyCode::Char('x') => {
            if let Some(a) = app.selected_pad_addr() {
                match pads::forget(&a) {
                    Ok(m) => {
                        app.ok = true;
                        app.msg = m;
                    }
                    Err(e) => {
                        app.ok = false;
                        app.msg = e;
                    }
                }
                app.refresh();
            }
        }
        _ => {}
    }
    false
}

fn source(app: &mut App, name: &str) {
    if let Some(n) = app.selected_name() {
        app.run_heos(&["source", name, "--speaker", &n]);
        match heos::match_source(name) {
            Ok(m) if !m.is_empty() => {
                app.msg = format!("{} · {m}", app.msg);
            }
            Ok(_) => {}
            Err(e) => {
                app.msg = format!("{} · match: {e}", app.msg);
            }
        }
    }
}

fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(4),
            Constraint::Length(4),
        ])
        .split(f.area());
    let tabs: String = FACES
        .iter()
        .enumerate()
        .map(|(i, fc)| {
            if i == app.face {
                format!("[{}]", fc.title())
            } else {
                format!(" {} ", fc.title())
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!(" wisp  {tabs}"),
            Style::default().add_modifier(Modifier::BOLD),
        )))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("lanterns · speakers"),
        ),
        chunks[0],
    );
    let body = match app.face() {
        Face::Speakers | Face::Source => speakers_body(app),
        Face::Enlist => enlist_body(app),
        Face::Ap => ap_body(),
        Face::Pads => pads_body(app),
    };
    f.render_widget(
        Paragraph::new(body).block(
            Block::default()
                .borders(Borders::ALL)
                .title(app.face().title()),
        ),
        chunks[1],
    );
    let hint = match app.face() {
        Face::Speakers => {
            "j/k select · +/- vol · m mute · p play · s stop · g/u group · r refresh · q quit"
        }
        Face::Source => "a AUX · b bluetooth · n network (siren follow/radio) · j/k pick box",
        Face::Enlist => "raise AP (Ap face, A) · WPS: Connect ~3s on the speaker · or one RJ45",
        Face::Ap => "A raise hotspot · Z lower · via ether ap (iw + hostapd; no NetworkManager)",
        Face::Pads => {
            "s scan · enter pair · c connect · d disconnect · x forget · unplug USB-C · SHARE+PS"
        }
    };
    let msg_style = if app.ok {
        Style::default()
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    };
    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(hint, Style::default())),
            Line::from(Span::styled(format!("  {}", app.msg), msg_style)),
        ])
        .block(Block::default().borders(Borders::ALL).title("menu")),
        chunks[2],
    );
}

fn speakers_body(app: &App) -> Vec<Line<'static>> {
    if app.players.is_empty() {
        return vec![Line::from("  none yet — Tab to enlist")];
    }
    let mut lines = Vec::new();
    if app.players.len() == 1 {
        lines.push(Line::from(
            "  one box (wired singleton). AP is how a fleet groups.",
        ));
    }
    for (i, p) in app.players.iter().enumerate() {
        let mark = if i == app.sel { ">" } else { " " };
        let name = p.get("name").and_then(|x| x.as_str()).unwrap_or("?");
        let model = p.get("model").and_then(|x| x.as_str()).unwrap_or("");
        let ip = p.get("ip").and_then(|x| x.as_str()).unwrap_or("");
        let st = p.get("state").and_then(|x| x.as_str()).unwrap_or("?");
        let vol = p
            .get("volume")
            .and_then(|x| x.as_i64())
            .map(|v| format!(" {v}%"))
            .unwrap_or_default();
        lines.push(Line::from(format!(
            " {mark} {name}  {model}  {ip}  {st}{vol}"
        )));
    }
    if app.face() == Face::Source {
        lines.push(Line::from(""));
        lines.push(Line::from("  a AUX  ·  b Bluetooth  ·  n network"));
    }
    lines
}

fn enlist_body(app: &App) -> Vec<Line<'static>> {
    vec![
        Line::from("  HEOS CLI needs IPv4 to the box (door 1255). No Denon app."),
        Line::from(""),
        Line::from("  1. Tab to ap, press A — house hotspot (ether ap on; SSID from ap.env)."),
        Line::from("  2. On the speaker: Connect ~3 seconds (WPS)."),
        Line::from("  3. One RJ45 is a singleton foothold — grouping needs the AP."),
        Line::from(""),
        Line::from(format!("  roster now: {} box(es)", app.players.len())),
        Line::from("  r refresh after the LED goes solid."),
    ]
}

fn ap_body() -> Vec<Line<'static>> {
    vec![
        Line::from(format!("  {}", ap::status_line())),
        Line::from(""),
        Line::from("  A  raise   (ether ap on)"),
        Line::from("  Z  lower   (ether ap off)"),
        Line::from("  PSK is never printed. ether reads ~/.config/ether/ap.env (0600)."),
    ]
}

fn pads_body(app: &App) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(
            "  Battletron USB identity is Nacon GC-100 (xpad). Cable is play, not charge-only.",
        ),
        Line::from("  Wireless: unplug, s scan, hold SHARE+PS (or Home) until it blinks."),
        Line::from(""),
    ];
    if app.pads.is_empty() {
        lines.push(Line::from("  no pads yet"));
        return lines;
    }
    for (i, p) in app.pads.iter().enumerate() {
        let mark = if i == app.pad_sel { ">" } else { " " };
        let name = p.get("name").and_then(|x| x.as_str()).unwrap_or("pad");
        let addr = p.get("addr").and_then(|x| x.as_str()).unwrap_or("");
        let js = p
            .get("js")
            .and_then(|x| x.as_str())
            .map(|s| format!(" {s}"))
            .unwrap_or_default();
        let conn = if p
            .get("connected")
            .and_then(|x| x.as_bool())
            .unwrap_or(false)
        {
            " connected"
        } else {
            ""
        };
        lines.push(Line::from(format!(
            " {mark} {name}  {}{conn}{js}",
            pads::trunc_addr(addr)
        )));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn faces_cover_the_join_paths() {
        let src = std::fs::read_to_string("src/tui.rs").expect("tui.rs");
        let prod = src.split("mod tests").next().unwrap();
        for n in ["Speakers", "Source", "Enlist", "Ap", "Pads"] {
            assert!(prod.contains(n), "face {n}");
        }
        assert!(prod.contains("wired singleton"));
        assert!(prod.contains("source"));
        assert!(prod.contains("SHARE+PS"));
        assert!(!prod.contains("sign_in"));
        assert!(!prod.contains("bluetoothctl"));
    }

    #[test]
    fn shared_cycle_bindings_wrap_faces_in_both_directions() {
        let mut app = App {
            face: 0,
            sel: 0,
            players: Vec::new(),
            pads: Vec::new(),
            pad_sel: 0,
            msg: String::new(),
            ok: true,
        };

        assert!(!handle(&mut app, KeyCode::BackTab, KeyModifiers::NONE));
        assert_eq!(app.face, FACES.len() - 1);
        assert!(!handle(&mut app, KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(app.face, 0);
        assert!(!handle(&mut app, KeyCode::Tab, KeyModifiers::CONTROL));
        assert_eq!(app.face, 0);
    }
}
