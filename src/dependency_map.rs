use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    ffi::OsStr,
    path::Path,
};

use color_eyre::eyre::OptionExt;
use petgraph::algo::{dominators::simple_fast, has_path_connecting, DfsSpace};

use crate::{graph::Graph, nix_cli::FlakeOutputDerivation};

pub type DependencyMap = BTreeMap<Derivation, BTreeSet<Derivation>>;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Hash)]
pub struct Derivation {
    pub drv_path: String,
    pub flake_path: String,
    pub drv_name: String,
}

impl TryFrom<FlakeOutputDerivation> for Derivation {
    type Error = color_eyre::eyre::Error;

    fn try_from(value: FlakeOutputDerivation) -> std::result::Result<Self, Self::Error> {
        Ok(Derivation {
            drv_name: AsRef::<Path>::as_ref(&value.drv_path)
                .file_name()
                .and_then(OsStr::to_str)
                .map(str::to_string)
                .ok_or_eyre("Could not get string from drv_path")?,
            drv_path: value.drv_path,
            flake_path: value.flake_path,
        })
    }
}

pub struct DependencyMapBuilder<'a> {
    pub graph: &'a Graph,
    pub dependencies: Vec<Derivation>,
    /// Needs to be equivalent to the flake_path
    pub skipped_dependencies: Vec<String>,
    /// Filter out all dependencies which are only required due to other dependencies
    pub skip_dominated: bool,
}

impl DependencyMapBuilder<'_> {
    fn build_map(&self) -> DependencyMap {
        let mut dfs = DfsSpace::new(self.graph);
        let mut map = BTreeMap::new();
        for dependent in self
            .dependencies
            .iter()
            .filter(|d| !self.skipped_dependencies.contains(&d.flake_path))
        {
            let mut set = BTreeSet::new();
            for candidate in self
                .dependencies
                .iter()
                .filter(|c| c.ne(&dependent))
                .filter(|c| !self.skipped_dependencies.contains(&c.flake_path))
            {
                // candidate and dependent are swapped, because the graph is directed from the dependencies to dependent
                if has_path_connecting(
                    self.graph,
                    &candidate.drv_name,
                    &dependent.drv_name,
                    Some(&mut dfs),
                ) {
                    set.insert(candidate.clone());
                }
            }
            map.insert(dependent.clone(), set);
        }
        map
    }

    /// Filter out all dependencies which are only indirectly required by a dependent
    ///
    /// Dependencies are only indirectly required, if they are dominated by any other dependency of a dependent
    fn filter_dominated(&self, map: DependencyMap) -> DependencyMap {
        // cache dominators for all checked dependencies in a HashMap
        let mut dominators = HashMap::new();
        let mut new_map = BTreeMap::new();
        for (dependent, dependencies) in map.iter() {
            let mut new_dependencies = BTreeSet::new();
            for dependency in dependencies.iter() {
                // get dominators for the root node of the current dependency
                let dominator = dominators
                    .entry(dependency.clone())
                    .or_insert_with(|| simple_fast(self.graph, &dependency.drv_name));
                // find strict dominators for the current dependent to the current dependency
                let mut dominator = dominator
                    .strict_dominators(&dependent.drv_name)
                    .expect("observed depencencies must be reachable here");
                // the current dependency is only indirectly required, if there is any other dependency which dominates the dependent
                let is_dominated = dominator.any(|dom| {
                    dependencies
                        .iter()
                        .filter(|dep| dep.ne(&dependency))
                        .any(|dep| dep.drv_name.eq(&dom))
                });
                if !is_dominated {
                    new_dependencies.insert(dependency.clone());
                }
            }
            new_map.insert(dependent.clone(), new_dependencies);
        }

        new_map
    }

    pub fn build(&self) -> DependencyMap {
        let mut map = self.build_map();
        if self.skip_dominated {
            map = self.filter_dominated(map);
        }
        map
    }
}
