use color_eyre::eyre::Result;
use graph::{Graph, GraphExt};

mod graph;

fn main() -> Result<()> {
    colog::init();
    color_eyre::install()?;

    let mut graph = Graph::new();

    let path = "/nix/store/yn1fkbzqij1wqsj6v0fhgpw0k0dwx102-microkit-sdk-1.4.1.drv";
    graph.extend_from_store_path(path)?;

    let path = "/nix/store/pkjvdd070h5rggfpk9zvdsxyqil8q67c-arm-trusted-firmware-zynqmp-aarch64-unknown-linux-gnu-xilinx-v2023.2.drv";
    graph.extend_from_store_path(path)?;

    println!("{}", graph.edge_count());

    Ok(())
}
