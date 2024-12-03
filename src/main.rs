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
        problem.nodes.intervals.iter().map(|s| s[0]).min(),
        problem.nodes.intervals.iter().map(|s| s[1]).max(),
        problem.nodes.costs.iter().map(Vec::len).sum::<usize>()
    );
    println!(
        "  {} edges, {} connections",
        problem.edges.costs.iter().map(Vec::len).sum::<usize>(),
        problem.edges.nodes.len()
    );

    let mut model = model::Model::new(problem.name);
    println!("{model}");
    fs::write(
        Path::new(&data_file).with_extension("mps"),
        model.to_string(),
    )?;
    Ok(())
}
