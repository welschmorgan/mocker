use std::{
  collections::VecDeque,
  net::{IpAddr, Shutdown, TcpListener, TcpStream},
  sync::Arc,
  thread,
};

use clap::{Parser, Subcommand};
use mocker::{Response, Workspace, CONFIG_NAME};
use std::io::Write;

#[derive(Subcommand)]
enum Command {
  /// Initialize the current workspace
  Init {},
  /// Serve the current workspace
  Serve {},
}

#[derive(Parser)]
#[command(version, about, long_about)]
struct Options {
  #[command(subcommand)]
  command: Command,
}

fn cmd_init() -> mocker::Result<()> {
  let w = Workspace::create(CONFIG_NAME)?;
  Ok(())
}

fn cmd_serve() -> mocker::Result<()> {
  let w = Workspace::load(CONFIG_NAME)?;
  println!("Workspace: {:#?}", w);
  let srv = Server::new(w.config.host, w.config.port);
  srv.listen();
  Ok(())
}

fn run() -> mocker::Result<()> {
  let options = Options::parse();
  match options.command {
    Command::Init { .. } => cmd_init(),
    Command::Serve { .. } => cmd_serve(),
  }
}

fn main() {
  if let Err(e) = run() {
    eprintln!("\x1b[1;31mfatal\x1b[0m: {}", e);
  }
}
