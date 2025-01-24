use std::{collections::BTreeSet, process::Command};

use file_lock::{FileLock, FileOptions};

pub struct Derivation {
    flake_url: String,
    attribute_path: String,
}

impl Derivation {
    pub fn depends_on(&self, other: &Self) -> bool {
        let output = Command::new("nix")
            .arg("why-depends")
            .arg("--quiet")
            .arg("--derivation")
            .arg("--")
            .arg(self.to_string())
            .arg(other.to_string())
            .output();

        !output.unwrap().stdout.is_empty()
    }

    pub fn eval_to_drv(&self) -> String {
        let output = Command::new("nix")
            .arg("eval")
            .arg("--json")
            .arg("--")
            .arg(self.to_string())
            .output();
        String::from_utf8_lossy(&output.unwrap().stdout).to_string()
    }
}

impl std::fmt::Display for Derivation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            flake_url,
            attribute_path,
        } = self;
        write!(f, "{flake_url}#{attribute_path}")
    }
}

type NixDependencyCache = BTreeSet<(Derivation, Derivation, bool)>;

pub struct CachedDependencyChecker {}

impl CachedDependencyChecker {
    pub fn depends_on(&mut self, subject: &Derivation, maybe_dep: &Derivation) -> bool {
        let cache_dir = dirs::cache_dir().unwrap();
        let file_name = "";

        let should_we_block = true;
        let options = FileOptions::new().write(true).create(true).append(true);

        let mut filelock = match FileLock::lock("myfile.txt", should_we_block, options) {
            Ok(lock) => lock,
            Err(err) => panic!("Error getting write lock: {}", err),
        };

        todo!();
    }
}
