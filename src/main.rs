#![allow(dead_code)]
use std::{env::args, fs};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Data {
    problem: Problem,
}

#[derive(Debug, Deserialize)]
struct Problem {
    name: String,
    nodes: ProblemNodes,
    edges: ProblemEdges,
    usage_limit: u64,
}

#[derive(Debug, Deserialize)]
struct ProblemNodes {
    intervals: Vec<[u64; 2]>,
    costs: Vec<Vec<u64>>,
    usages: Vec<Vec<u64>>,
}

#[derive(Debug, Deserialize)]
struct ProblemEdges {
    nodes: Vec<[usize; 2]>,
    costs: Vec<Vec<u64>>,
}

fn main() -> anyhow::Result<()> {
    let Some(data_file) = args().nth(1) else {
        println!("Specify data file");
        return Ok(());
    };
    let problem = serde_json::from_slice::<Data>(&fs::read(data_file)?)?.problem;
    println!("Problem {}", problem.name);
    println!(
        "  {} nodes, interval min {:?} max {:?}, {} total strategies",
        problem.nodes.costs.len(),
        problem.nodes.intervals.iter().map(|s| s[0]).min(),
        problem.nodes.intervals.iter().map(|s| s[1]).max(),
        problem.nodes.costs.iter().map(Vec::len).sum::<usize>()
    );
    println!(
        "  {} edges, {} connections",
        problem.edges.costs.iter().map(Vec::len).sum::<usize>(),
        problem.edges.nodes.len()
    );
    Ok(())
}
