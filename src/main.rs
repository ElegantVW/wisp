//! wisp — house lanterns and HEOS remote. No vendor cloud.

mod ap;
mod grove_keys;
mod heos;
mod pads;
mod paths;
mod tui;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wisp", version, about = "house lanterns · HEOS remote")]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// One-shot roster
    Status,
    /// Refresh roster through siren
    Discover,
    /// House AP
    Ap {
        #[command(subcommand)]
        sub: ApCmd,
    },
    /// Gamepads via ether bt
    Pads {
        #[command(subcommand)]
        sub: Option<PadsCmd>,
    },
}

#[derive(Subcommand)]
enum PadsCmd {
    Status,
    Scan,
}

#[derive(Subcommand)]
enum ApCmd {
    On,
    Off,
    Status,
}

fn main() {
    let cli = Cli::parse();
    let code = match cli.cmd {
        None => match tui::run() {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("wisp tui: {e:#}");
                1
            }
        },
        Some(Cmd::Status) | Some(Cmd::Discover) => match heos::cmd(&["list", "--live"]) {
            Ok(v) => {
                println!("{v}");
                0
            }
            Err(e) => {
                eprintln!("{e}");
                1
            }
        },
        Some(Cmd::Ap { sub }) => match sub {
            ApCmd::Status => {
                println!("{}", ap::status_line());
                0
            }
            ApCmd::On => match ap::raise() {
                Ok(m) => {
                    println!("{m}");
                    0
                }
                Err(e) => {
                    eprintln!("{e}");
                    1
                }
            },
            ApCmd::Off => match ap::lower() {
                Ok(m) => {
                    println!("{m}");
                    0
                }
                Err(e) => {
                    eprintln!("{e}");
                    1
                }
            },
        },
        Some(Cmd::Pads { sub }) => match sub.unwrap_or(PadsCmd::Status) {
            PadsCmd::Status => {
                println!("{}", pads::status_line());
                0
            }
            PadsCmd::Scan => match pads::scan() {
                Ok(v) => {
                    println!("{v}");
                    0
                }
                Err(e) => {
                    eprintln!("{e}");
                    1
                }
            },
        },
    };
    std::process::exit(code);
}
