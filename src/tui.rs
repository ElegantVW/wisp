//! Four faces: speakers · source · enlist · ap.

use crate::{ap, heos};
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
}

const FACES: [Face; 4] = [Face::Speakers, Face::Source, Face::Enlist, Face::Ap];

impl Face {
    fn title(self) -> &'static str {
        match self {
            Face::Speakers => "speakers",
            Face::Source => "source",
            Face::Enlist => "enlist",
            Face::Ap => "ap",
        }
    }
}

struct App {
    face: usize,
    sel: usize,
    players: Vec<Value>,
    msg: String,
    ok: bool,
}

impl App {
    fn new() -> Self {
        let mut a = Self {
            face: 0,
            sel: 0,
            players: Vec::new(),
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

fn handle(app: &mut App, code: KeyCode, _mods: KeyModifiers) -> bool {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => return true,
        KeyCode::Tab => app.face = (app.face + 1) % FACES.len(),
        KeyCode::BackTab => app.face = (app.face + FACES.len() - 1) % FACES.len(),
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
        .constraints([Constraint::Length(3), Constraint::Min(4), Constraint::Length(4)])
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
        .block(Block::default().borders(Borders::ALL).title("lanterns · speakers")),
        chunks[0],
    );
    let body = match app.face() {
        Face::Speakers | Face::Source => speakers_body(app),
        Face::Enlist => enlist_body(app),
        Face::Ap => ap_body(),
    };
    f.render_widget(
        Paragraph::new(body).block(Block::default().borders(Borders::ALL).title(app.face().title())),
        chunks[1],
    );
    let hint = match app.face() {
        Face::Speakers => "j/k select · +/- vol · m mute · p play · s stop · g/u group · r refresh · q quit",
        Face::Source => "a AUX · b bluetooth · n network (siren follow/radio) · j/k pick box",
        Face::Enlist => "raise AP (Ap face, A) · WPS: Connect ~3s on the speaker · or one RJ45",
        Face::Ap => "A raise hotspot · Z lower · via ether ap (iw + hostapd; no NetworkManager)",
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
        lines.push(Line::from("  one box (wired singleton). AP is how a fleet groups."));
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

#[cfg(test)]
mod tests {
    #[test]
    fn faces_cover_the_join_paths() {
        let src = std::fs::read_to_string("src/tui.rs").expect("tui.rs");
        let prod = src.split("mod tests").next().unwrap();
        for n in ["Speakers", "Source", "Enlist", "Ap"] {
            assert!(prod.contains(n), "face {n}");
        }
        assert!(prod.contains("wired singleton"));
        assert!(prod.contains("source"));
        assert!(!prod.contains("sign_in"));
    }
}
