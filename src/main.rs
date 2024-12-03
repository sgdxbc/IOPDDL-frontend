use std::{env::args, fs, path::Path};

mod data;
mod model;

fn main() -> anyhow::Result<()> {
    let Some(data_file) = args().nth(1) else {
        println!("Specify data file");
        return Ok(());
    };
    let problem = serde_json::from_slice::<data::Data>(&fs::read(&data_file)?)?.problem;
    println!("Problem {}", problem.name);
    println!(
        "  {} nodes, interval min {:?} max {:?}, {} total strategies",
        problem.nodes.costs.len(),
        problem.nodes.intervals.iter().map(|s| s.0).min(),
        problem.nodes.intervals.iter().map(|s| s.1).max(),
        problem.nodes.costs.iter().map(Vec::len).sum::<usize>()
    );
    println!(
        "  {} edges, {} connections",
        problem.edges.costs.iter().map(Vec::len).sum::<usize>(),
        problem.edges.nodes.len()
    );

    let mut model = model::Model::new(problem.name);
    let mut strategy_vars = Vec::new();
    struct StrategyVar {
        var_index: usize,
        cost: u64,
        interval: (u64, u64),
        usage: u64,
    }
    for (node_index, node_costs) in problem.nodes.costs.iter().enumerate() {
        let mut node_strategy_vars = Vec::new();
        for (strategy_index, cost) in node_costs.iter().copied().enumerate() {
            let var_index = model.add_var(format!("n{node_index:x}_{strategy_index:x}"))?;
            node_strategy_vars.push(StrategyVar {
                var_index,
                cost,
                interval: problem.nodes.intervals[node_index],
                usage: problem.nodes.usages[node_index][strategy_index],
            });
        }
        strategy_vars.push(node_strategy_vars);
    }

    for (node_index, node_strategy_vars) in strategy_vars.iter().enumerate() {
        let mut cols = model::Cols::new();
        for var in node_strategy_vars {
            cols.push(1, var.var_index)
        }
        model.add_constr(model::Constr {
            name: Some(format!("U{node_index:x}")), // U for unique selection
            cols,
            typ: model::ConstrType::Equal,
            rhs: 1,
        })?
    }

    // println!("{model}");
    fs::write(
        Path::new(&data_file).with_extension("mps"),
        model.to_string(),
    )?;
    Ok(())
}
