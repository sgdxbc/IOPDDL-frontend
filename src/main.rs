use std::{
    collections::BTreeMap,
    env::args,
    fs::{self, File},
    io::{BufWriter, Write as _},
    path::Path,
};

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
        "  {} nodes, {} total strategies, interval min {:?} max {:?}",
        problem.nodes.costs.len(),
        problem.nodes.costs.iter().map(Vec::len).sum::<usize>(),
        problem.nodes.intervals.iter().map(|s| s.0).min(),
        problem.nodes.intervals.iter().map(|s| s.1).max(),
    );
    println!(
        "  {} edges, {} connections",
        problem.edges.nodes.len(),
        problem.edges.costs.iter().map(Vec::len).sum::<usize>(),
    );

    let mut model = model::Model::new(problem.name);
    let mut obj = model::Cols::new();

    println!("Node primary enumeration");
    // {node -> {strategy -> var}}
    let mut strategy_vars = Vec::new();
    // interval.0 -> [node]
    let mut instant_nodes = BTreeMap::new();
    let mut var_name = {
        let mut count = 0;
        move || {
            count += 1;
            format!("S{count:07}")
        }
    };
    assert_eq!(problem.nodes.costs.len(), problem.nodes.intervals.len());
    for (node_index, (node_costs, node_interval)) in problem
        .nodes
        .costs
        .iter()
        .zip(&problem.nodes.intervals)
        .enumerate()
    {
        let mut node_strategy_vars = Vec::new();
        let mut cols = model::Cols::new();
        for (strategy_index, &cost) in node_costs.iter().enumerate() {
            let var_index = model.add_var(
                var_name(),
                Some(format!(
                    "Decision varaible for strategy {node_index}/{strategy_index}"
                )),
            )?;
            // contribute to objective
            obj.push(cost.try_into()?, var_index);
            // contribute to node strategy selection (below)
            cols.push(1, var_index);
            // bookkeeping
            node_strategy_vars.push(var_index)
        }

        model.add_constr(model::Constr {
            name: format!("U{node_index:07x}"), // U for unique selection
            #[cfg(feature = "commented-model")]
            desc: Some(format!(
                "Exact one strategy is select for node {node_index}"
            )),
            cols,
            typ: model::ConstrType::Equal,
            rhs: 1,
        })?;

        instant_nodes.insert(node_interval.0, Vec::new());
        strategy_vars.push(node_strategy_vars)
    }

    println!("Node interval enumeration");
    for (node_index, &(start, end)) in problem.nodes.intervals.iter().enumerate() {
        for (_, nodes) in instant_nodes.range_mut(start..end) {
            nodes.push(node_index)
        }
    }
    for (instant, nodes) in instant_nodes {
        let mut cols = model::Cols::new();
        for node_index in nodes {
            assert_eq!(
                strategy_vars[node_index].len(),
                problem.nodes.usages[node_index].len()
            );
            for (&var_index, &usage) in strategy_vars[node_index]
                .iter()
                .zip(&problem.nodes.usages[node_index])
            {
                cols.push(usage.try_into()?, var_index)
            }
        }
        model.add_constr(model::Constr {
            name: format!("I{instant:07}"), // I for interval usage
            #[cfg(feature = "commented-model")]
            desc: Some(format!("Usage limit at instant {instant}")),
            cols,
            typ: model::ConstrType::LessEqual,
            rhs: problem.usage_limit.try_into()?,
        })?
    }

    println!("Edge enumeration");
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
            format!("C{count:07x}") // C for connectivity
        }
    };
    assert_eq!(problem.edges.nodes.len(), problem.edges.costs.len());
    for (&(node_v, node_u), edge_costs) in problem.edges.nodes.iter().zip(&problem.edges.costs) {
        assert_eq!(
            edge_costs.len(),
            strategy_vars[node_v].len() * strategy_vars[node_u].len()
        );
        let mut i = 0;
        for (v_index, &v_strategy_var) in strategy_vars[node_v].iter().enumerate() {
            for (u_index, &u_strategy_var) in strategy_vars[node_u].iter().enumerate() {
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
                cols.push(-1, v_strategy_var);
                model.add_constr(model::Constr {
                    name: constr_name(),
                    #[cfg(feature = "commented-model")]
                    desc: Some(format!("{edge_desc} >= {node_v}/{v_index}")),
                    cols,
                    typ: model::ConstrType::GraterEqual,
                    rhs: 0,
                })?;
                // edge >= strategy u i.e. edge must be selected if strategy u is selected
                let mut cols = model::Cols::new();
                cols.push(1, var_index);
                cols.push(-1, u_strategy_var);
                model.add_constr(model::Constr {
                    name: constr_name(),
                    #[cfg(feature = "commented-model")]
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
    println!("Write model file");
    let mut model_file = BufWriter::new(File::create(Path::new(&data_file).with_extension("mps"))?);
    write!(&mut model_file, "{model}")?;
    Ok(())
}
