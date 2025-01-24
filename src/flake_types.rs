use std::{char::DecodeUtf16Error, collections::HashMap, process::Command};

use petgraph::Graph;
use serde_json::Value;

pub type StringMap<T> = HashMap<String, T>;
// type Flake = StringMap<StringMap<AttrsOrDerivation>>;

enum AttrsOrDerivation {
    Attrs(StringMap<DerivationValue>),
    DerivationValue(DerivationValue),
}

#[derive(serde::Deserialize)]
struct DerivationValue {
    name: String,
    r#type: String,
    description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Derivation {
    pub name: String,
    pub description: Option<String>,
    pub kind: String,
    pub system: String,
    pub attr: String,
}

impl Derivation {
    pub fn attr_path(&self) -> String {
        let Self {
            kind, system, attr, ..
        } = self;
        format!("{kind}.{system}.{attr}")
    }
}

/// Opens a flake via URL, and enumerates all derivations exposed by that flake
///
/// In particular, this will provide a list of `packages` & `checks`
// TODO what about other outputs
pub fn derivations_from_url(url: &str) -> Vec<Derivation> {
    // run command and parse output JSON
    let output = Command::new("nix")
        .arg("flake")
        .arg("show")
        .arg("--all-systems")
        .arg("--json")
        .arg("--")
        .arg(url)
        .output();

    let string = String::from_utf8(output.unwrap().stdout).unwrap();

    let flake: StringMap<serde_json::Value> = serde_json::from_str(&string).unwrap();

    // collect interesting bits and pieces
    let mut vec = Vec::new();

    for (kind, systems) in flake.into_iter() {
        // filter out unwanted outputs
        if !["checks", "packages"].contains(&kind.as_str()) {
            continue;
        }

        // we expect an JSON object (collection of attributes), containing the systems (x86_64-linux, ...) for the current kind (packages, devShells, ...)
        let Value::Object(systems) = systems else {
            continue;
        };

        for (system, attrs) in systems.into_iter() {
            let Value::Object(attrs) = attrs else {
                continue;
            };
            for (attr, drv_val) in attrs.into_iter() {
                let Value::Object(drv_val) = drv_val else {
                    continue;
                };

                vec.push(Derivation {
                    kind: kind.clone(),
                    system: system.clone(),
                    name: drv_val.get("name").unwrap().as_str().unwrap().to_owned(),
                    description: drv_val
                        .get("description")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                    attr,
                })
            }
        }
    }

    vec
}

/// Turns a flake URL to a derivation (e.g. github:this/that#my-package) and evaluates the Nix expression, yielding a derivation path (e.g. /nix/store/*-something.drv)
pub fn get_derivation_path(url: &str) -> String {
    let output = Command::new("nix")
        .arg("build")
        .arg("--dry-run")
        .arg("--json")
        .arg("--")
        .arg(url)
        .output();

    let string = String::from_utf8(output.unwrap().stdout).unwrap();

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct DerivationPath {
        drv_path: String,
    }

    println!("{url} => {string}");

    let derivation_paths: Vec<DerivationPath> = serde_json::from_str(&string).unwrap();

    if derivation_paths.len() != 1 {
        panic!("{derivation_paths:?}")
    }

    derivation_paths[0].drv_path.clone()
}

pub fn drv_to_transitive_deps(drv_path: &str) -> dot_parser::canonical::Graph<()> {
    use dot_parser::*;
    let output = Command::new("nix-store")
        .arg("--query")
        .arg("--graph") // dot gaph out
        .arg("--")
        .arg(drv_path)
        .output();

    let string = String::from_utf8(output.unwrap().stdout).unwrap();

    let ast_dot_graph = ast::Graph::try_from(string.as_str()).unwrap();
    // .filter_map(&|(_, _)| Some(()));

    // let canonical_dot_graph = canonical::Graph::from(ast_dot_graph);

    // for edge in &dot_graph.edges.set {
    //     println!("{} -> {}", edge.from, edge.to);
    // }

    // canonical_dot_graph
    todo!();

    // use graph.add_edge, which updates the graph as well

    // let pet_graph: Graph<dot_parser::canonical::Node<_>, dot_parser::canonical::AList<_>> =
    // dot_graph.into();

    // let mut node_idx_map = HashMap::new();

    // let pet_graph = pet_graph.map(
    //     |node_idx, node| {
    //         let node_name = node.id;
    //         node_idx_map.insert(node_name, node_id);
    //         node_name
    //     },
    //     |edge_idx, edge| edge.elem,
    // );
}
