use std::path::PathBuf;

use clap::Parser;
use regex::Regex;
use tracing::level_filters::LevelFilter;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    dataset_path: String,

    #[arg(short, long, default_value = "4")]
    thread: usize,
}

#[derive(Debug, Clone)]
struct MetaData {
    region: u32,
    row: u32,
    col: u32,
}

fn main() {
    let indicatif_layer = IndicatifLayer::new();
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_writer(indicatif_layer.get_stderr_writer())
        .with_level(true);
    let filter_layer = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(indicatif_layer)
        .init();

    let cli = Cli::parse();

    rayon::ThreadPoolBuilder::new()
        .num_threads(cli.thread.into())
        .build_global()
        .unwrap();

    let re = Regex::new(r"Tile_r(\d+)-c(\d+)_Region(\d+)").unwrap();
    let mut files: Vec<(PathBuf, MetaData)> = vec![];
    for entry in std::fs::read_dir(cli.dataset_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let cap = re.captures(path.to_str().unwrap()).unwrap();
            let region = cap.get(3).unwrap().as_str().parse().unwrap();
            let row = cap.get(1).unwrap().as_str().parse().unwrap();
            let col = cap.get(2).unwrap().as_str().parse().unwrap();
            files.push((entry.path(), MetaData { region, row, col }));
        }
    }
    tracing::debug!("files: {:?}", files);
}
