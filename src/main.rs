use std::{
  collections::VecDeque,
  net::{IpAddr, Shutdown, TcpListener, TcpStream},
  sync::Arc,
  thread,
};

use clap::{Parser, Subcommand};
use mocker_core::{Response, Server, Workspace, CONFIG_NAME};
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

fn cmd_init() -> mocker_core::Result<()> {
  let w = Workspace::create(CONFIG_NAME)?;
  println!("{:#?}", w);
  Ok(())
}

fn cmd_serve() -> mocker_core::Result<()> {
  let w = Workspace::load(CONFIG_NAME)?;
  println!("{:#?}", w);
  let srv = Server::new(w.config);
  srv.listen()?;
  Ok(())
}

fn run() -> mocker_core::Result<()> {
  let options = Options::parse();
  if let Err(_) = std::env::var("RUST_LOG") {
    std::env::set_var("RUST_LOG", "info");
  }
  pretty_env_logger::init();
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
