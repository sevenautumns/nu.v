use std::process::{Command, Stdio};

use color_eyre::eyre::{bail, Result};
use log::{error, info, trace};

/// A magic Nix command to get all relevant outputs of a flake
const MAGIC_NIX_SCRIPT: &str = include_str!("../assets/get-flake-output-drvs-paths.nix");

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlakeOutputDerivation {
    pub drv_path: String,
    pub flake_path: String,
}

/// Gather all output derivations of a flake specified via flak url
pub fn get_flake_output_derivations(flake_url: &str) -> Result<Vec<FlakeOutputDerivation>> {
    info!("evaluating entire flake, this might take a couple of minutes");

    let output = Command::new("nix")
        .args([
            "eval",
            "--json",
            "--include",
            "nixpkgs=flake:nixpkgs",
            "--impure",
            "--apply",
            MAGIC_NIX_SCRIPT,
        ])
        .arg(format!("{flake_url}#."))
        .stdin(Stdio::null())
        .output()?;
    info!("done evaluating, parsing results");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!(
            "flake_url = {flake_url:?}, {}, stderr:\n{stderr}",
            output.status
        );
        bail!("nix eval command failed with {}", output.status)
    }

    let flake_outputs: Vec<FlakeOutputDerivation> = serde_json::from_slice(&output.stdout)?;
    trace!("found the following outputs:\n{flake_outputs:#?}");

    Ok(flake_outputs)
}

#[cfg(test)]
mod test {
    use super::get_flake_output_derivations;

    #[test_log::test]
    fn parse_sel4_nix_utils_flake() {
        let outputs = get_flake_output_derivations(".").unwrap();
        assert!(
            outputs.len() >= 6,
            "there surely are more than 6 packages in the nüv flake?!"
        )
    }
}
