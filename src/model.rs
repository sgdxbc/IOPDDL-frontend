use std::fmt::Display;

pub struct Model {
    name: String,
    vars: Vec<Var>,
    constrs: Vec<Constr>,
    objective: Cols,
}

pub struct Var {
    name: String,
    coes: Vec<(usize, i64)>,
    obj_coe: Option<i64>,
}

pub enum ConstrType {
    Equal,
    LessEqual,
    GraterEqual,
}

#[derive(Default)]
pub struct Cols(Vec<(i64, usize)>);

pub struct Constr {
    pub name: Option<String>,
    pub cols: Cols,
    pub typ: ConstrType,
    pub rhs: i64,
}

impl Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:14}{}", "NAME", self.name)?;

        const OBJ: &str = "OBJ";
        const EMPTY: &str = "";
        const IGNORED: &str = "_";

        writeln!(f, "ROWS")?;
        writeln!(f, "{EMPTY:1}{:3}{OBJ}", "N")?;
        for (i, constr) in self.constrs.iter().enumerate() {
            let typ = match &constr.typ {
                ConstrType::Equal => "E",
                ConstrType::LessEqual => "L",
                ConstrType::GraterEqual => "G",
            };
            let fallback_name = format!("R{i}");
            let name = constr.name.as_ref().unwrap_or(&fallback_name);
            writeln!(f, "{EMPTY:1}{typ:3}{name:}")?
        }

        writeln!(f, "COLUMNS")?;
        for var in &self.vars {
            if let Some(coe) = var.obj_coe {
                writeln!(f, "{EMPTY:4}{:10}{OBJ:10}{coe}", var.name)?
            }
            for &(constr_index, coe) in &var.coes {
                let fallback_name = format!("R{constr_index}");
                let constr_name = self.constrs[constr_index]
                    .name
                    .as_ref()
                    .unwrap_or(&fallback_name);
                writeln!(f, "{EMPTY:4}{:10}{constr_name:10}{coe}", var.name)?
            }
        }

        writeln!(f, "RHS")?;
        for (i, constr) in self.constrs.iter().enumerate() {
            let fallback_name = format!("R{i}");
            let name = constr.name.as_ref().unwrap_or(&fallback_name);
            writeln!(f, "{EMPTY:4}{IGNORED:10}{name:10}{}", constr.rhs)?
        }

        writeln!(f, "BOUNDS")?;
        for var in &self.vars {
            writeln!(f, "{EMPTY:1}{:3}{IGNORED:10}{}", "BV", var.name)?
        }

        write!(f, "ENDATA")
    }
}

impl Model {
    pub fn new(name: String) -> Self {
        Self {
            name,
            vars: Default::default(),
            constrs: Default::default(),
            objective: Default::default(),
        }
    }

    pub fn add_var(&mut self, name: String) -> anyhow::Result<usize> {
        anyhow::ensure!(name.len() < 10);
        let index = self.vars.len();
        self.vars.push(Var {
            name,
            coes: Default::default(),
            obj_coe: None,
        });
        Ok(index)
    }

    pub fn add_constr(&mut self, constr: Constr) -> anyhow::Result<()> {
        if let Some(name) = &constr.name {
            anyhow::ensure!(name.len() < 10)
        }
        let index = self.constrs.len();
        for &(coe, var_index) in &constr.cols.0 {
            self.vars[var_index].coes.push((index, coe))
        }
        self.constrs.push(constr);
        Ok(())
    }

    pub fn set_objective(&mut self, objective: Cols) {
        for &(coe, var_index) in &objective.0 {
            self.vars[var_index].obj_coe = Some(coe);
        }
        self.objective = objective
    }
}

impl Cols {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, coe: i64, var_index: usize) {
        self.0.push((coe, var_index))
    }
}
