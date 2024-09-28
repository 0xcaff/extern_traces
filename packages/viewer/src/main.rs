use crate::app::tracing::TracingScene;
use crate::app::Scene;
use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::path::PathBuf;

mod app;
mod proto;

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(name = "load")]
    LoadFile { path: PathBuf },

    #[command(name = "listen")]
    ListenNetwork { addr: SocketAddr },
}

fn main() -> Result<(), eframe::Error> {
    let args = Cli::parse();

    let scene = match args.command {
        None => Scene::initial(),
        Some(Commands::LoadFile { path }) => Scene::Tracing(TracingScene::from_file_path(path)),
        Some(Commands::ListenNetwork { addr }) => Scene::Tracing(TracingScene::from_network(addr)),
    };

    let options = eframe::NativeOptions {
        ..Default::default()
    };

    eframe::run_native(
        "extern_traces",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc, scene)))),
    )
}
