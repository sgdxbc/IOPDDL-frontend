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
    let mut obj = model::Cols::new();
    let mut strategy_vars = Vec::new();
    struct StrategyVar {
        var_index: usize,
        cost: u64,
        interval: (u64, u64),
        usage: u64,
    }
    let mut var_name = {
        let mut count = 0;
        move || {
            count += 1;
            format!("S{count:07}")
        }
    };
    for (node_index, ((node_costs, node_usages), &interval)) in problem
        .nodes
        .costs
        .iter()
        .zip(&problem.nodes.usages)
        .zip(&problem.nodes.intervals)
        .enumerate()
    {
        let mut node_strategy_vars = Vec::new();
        for (strategy_index, (&cost, &usage)) in node_costs.iter().zip(node_usages).enumerate() {
            let var_index = model.add_var(
                var_name(),
                Some(format!(
                    "Decision varaible for strategy {node_index}/{strategy_index}"
                )),
            )?;
            obj.push(cost.try_into()?, var_index);
            node_strategy_vars.push(StrategyVar {
                var_index,
                cost,
                interval,
                usage,
            })
        }
        strategy_vars.push(node_strategy_vars)
    }

    for (node_index, node_strategy_vars) in strategy_vars.iter().enumerate() {
        let mut cols = model::Cols::new();
        for var in node_strategy_vars {
            cols.push(1, var.var_index)
        }
        model.add_constr(model::Constr {
            name: format!("U{node_index:07}"), // U for unique selection
            desc: Some(format!(
                "Exact one strategy is select for node {node_index}"
            )),
            cols,
            typ: model::ConstrType::Equal,
            rhs: 1,
        })?
    }

    let mut var_name = {
        let mut count = 0;
        move || {
            count += 1;
            format!("E{count:07}")
        }
    };
    let mut constr_name = {
        let mut count = 0;
        move || {
            count += 1;
            format!("C{count:07}") // C for connectivity
        }
    };
    for (&(node_v, node_u), edge_costs) in problem.edges.nodes.iter().zip(&problem.edges.costs) {
        let mut i = 0;
        for (v_index, strategy_v) in strategy_vars[node_v].iter().enumerate() {
            for (u_index, strategy_u) in strategy_vars[node_u].iter().enumerate() {
                let cost = edge_costs[i];
                i += 1;

                let edge_desc = format!("{node_v}/{v_index}-{node_u}/{u_index}");
                let var_index = model.add_var(
                    var_name(),
                    Some(format!("Decision variable for edge {edge_desc}")),
                )?;
                // objective
                obj.push(cost.try_into()?, var_index);
                // edge >= strategy v i.e. edge must be selected if strategy v is selected
                let mut cols = model::Cols::new();
                cols.push(1, var_index);
                cols.push(-1, strategy_v.var_index);
                model.add_constr(model::Constr {
                    name: constr_name(),
                    desc: Some(format!("{edge_desc} >= {node_v}/{v_index}")),
                    cols,
                    typ: model::ConstrType::GraterEqual,
                    rhs: 0,
                })?;
                // edge >= strategy u i.e. edge must be selected if strategy u is selected
                let mut cols = model::Cols::new();
                cols.push(1, var_index);
                cols.push(-1, strategy_u.var_index);
                model.add_constr(model::Constr {
                    name: constr_name(),
                    desc: Some(format!("{edge_desc} >= {node_u}/{u_index}")),
                    cols,
                    typ: model::ConstrType::GraterEqual,
                    rhs: 0,
                })?
            }
        }
    }

    model.set_obj(obj);
    // println!("{model}");
    fs::write(
        Path::new(&data_file).with_extension("mps"),
        model.to_string(),
    )?;
    Ok(())
}
