use std::{
    ffi::OsStr,
    path::Path,
    process::{Command, Stdio},
    time::Instant,
};

use color_eyre::eyre::{bail, Result};
use log::{debug, error, trace};
use nom::{
    bytes::complete::{tag, take_until},
    character::complete::{char, space0},
    error::{ErrorKind, ParseError},
    sequence::{delimited, pair, preceded},
    IResult,
};
use petgraph::prelude::DiGraphMap;

pub type Graph = DiGraphMap<&'static str, ()>;

pub trait GraphExt {
    fn extend_from_store_path(&mut self, path: &str) -> Result<()>;
}

impl GraphExt for Graph {
    fn extend_from_store_path(&mut self, path: &str) -> Result<()> {
        let start_time = Instant::now();

        if AsRef::<Path>::as_ref(path)
            .file_name()
            .and_then(OsStr::to_str)
            .map(|f| self.contains_node(f))
            .unwrap_or_default()
        {
            debug!("path = {path}, graph already contains path. skipping");
            return Ok(());
        }

        let output = Command::new("nix-store")
            .args(["--query", path, "--graph"])
            .stdin(Stdio::null())
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("path = {path}, {}, stderr:\n{stderr}", output.status);
            bail!("nix-store command failed with {}", output.status)
        }

        let stdout = String::from_utf8(output.stdout)?.leak();
        let mut found_edges = 0;
        let mut new_edges = 0;

        for (_, (source, target)) in stdout.lines().filter_map(|l| parse_edge(l).ok()) {
            if self.add_edge(source, target, ()).is_none() {
                new_edges += 1
            };
            found_edges += 1;
        }

        trace!(
            "path = {path}, found_edges = {found_edges}, new_edges = {new_edges}, elapsed = {:?}",
            start_time.elapsed()
        );
        Ok(())
    }
}

pub fn node<'a, Error: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, Error> {
    preceded(space0, delimited(char('"'), take_until("\""), char('"')))(i)
}

pub fn arrow<'a, Error: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, Error> {
    preceded(space0, tag("->"))(i)
}

pub fn parse_edge(input: &str) -> IResult<&str, (&str, &str), (&str, ErrorKind)> {
    pair(node, preceded(arrow, node))(input)
}
