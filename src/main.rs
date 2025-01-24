use std::process::Command;

use rayon::iter::IntoParallelRefIterator;

use crate::flake_types::{drv_to_transitive_deps, get_derivation_path};

use rayon::prelude::*;

mod dependency_checker;
mod flake_types;

struct App {
    url: String,
}

struct Derivation {}

impl App {
    fn get_outputs(&mut self) {
        let jobs = &Command::new("nix")
            .args(&["eval", "json", &self.url, "--apply", "--quiet", "x: x.meta"])
            .status();
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let url = &args[1];

    let outputs = flake_types::derivations_from_url(&url);

    println!("{outputs:#?}");

    let total_dependency_graph = outputs.iter().for_each(|output| {
        println!("trying {url}");
        let drv_path = get_derivation_path(&format!("{url}#{}", output.attr_path()));
        println!("drv path is {drv_path}");
        let this_drv_transitive_deps = drv_to_transitive_deps(&drv_path);
    });
}
