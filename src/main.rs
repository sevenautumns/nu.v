use std::collections::BTreeSet;

use color_eyre::eyre::{OptionExt, Result};
use dependency_map::{DependencyMapBuilder, Derivation};
use graph::{Graph, GraphExt};
use log::info;

mod dependency_map;
mod graph;
mod nix_cli;

fn main() -> Result<()> {
    colog::init();
    color_eyre::install()?;

    let args: Vec<String> = std::env::args().collect();

    let that_flake = args
        .get(1)
        .ok_or_eyre(format!("usage: {} <flake-url>", env!("CARGO_BIN_NAME")))?;

    let flake_outputs = nix_cli::get_flake_output_derivations(that_flake)?
        .into_iter()
        .map(TryInto::<Derivation>::try_into)
        .collect::<Result<Vec<_>, _>>()?;

    let mut graph = Graph::new();

    info!("building graph");
    for flake_output in flake_outputs.iter() {
        graph.extend_from_store_path(&flake_output.drv_path)?;
    }

    info!("built graph with {} edges", graph.edge_count());

    let builder = DependencyMapBuilder {
        graph: &graph,
        dependencies: flake_outputs,
        skipped_dependencies: vec![],
        skip_dominated: false,
    };
    let deps = builder.build();

    let num_deps: usize = deps.values().map(BTreeSet::len).sum();
    info!("found a total of {num_deps} dependencies over all flake outputs");

    Ok(())
}
