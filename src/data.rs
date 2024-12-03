use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Data {
    pub problem: Problem,
}

#[derive(Debug, Deserialize)]
pub struct Problem {
    pub name: String,
    pub nodes: ProblemNodes,
    pub edges: ProblemEdges,
    pub usage_limit: u64,
}

#[derive(Debug, Deserialize)]
pub struct ProblemNodes {
    pub intervals: Vec<[u64; 2]>,
    pub costs: Vec<Vec<u64>>,
    pub usages: Vec<Vec<u64>>,
}

#[derive(Debug, Deserialize)]
pub struct ProblemEdges {
    pub nodes: Vec<[usize; 2]>,
    pub costs: Vec<Vec<u64>>,
}
