use color_eyre::eyre::{OptionExt, Result};
use graph::{Graph, GraphExt};
use log::info;

mod graph;
mod nix_cli;

fn main() -> Result<()> {
    colog::init();
    color_eyre::install()?;

    let args: Vec<String> = std::env::args().collect();

    let that_flake = args
        .get(1)
        .ok_or_eyre(format!("usage: {} <flake-url>", env!("CARGO_BIN_NAME")))?;

    let flake_outputs = nix_cli::get_flake_output_derivations(that_flake)?;

    let mut graph = Graph::new();

    info!("building graph");
    for flake_output in flake_outputs {
        graph.extend_from_store_path(&flake_output.drv_path)?;
    }

    info!("built the following graph: {}", graph.edge_count());

    Ok(())
}
