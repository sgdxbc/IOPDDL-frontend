use std::fmt::Display;

pub struct Model {
    name: String,
    vars: Vec<Var>,
    constrs: Vec<Constr>,
    obj: Cols,
}

pub struct Var {
    name: String,
    desc: Option<String>,
    nonzeros: Vec<(usize, i64)>, // (constr index, coefficient)
    obj_nonzero: Option<i64>,
}

pub enum ConstrType {
    Equal,
    LessEqual,
    GraterEqual,
}

#[derive(Default)]
pub struct Cols(Vec<(i64, usize)>); // (ceofficient, var index)

pub struct Constr {
    pub name: String,
    pub desc: Option<String>,
    pub cols: Cols,
    pub typ: ConstrType,
    pub rhs: i64,
}

impl Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:14}{}", "NAME", self.name)?;

        const OBJ: &str = "OBJECTIV";
        const EMPTY: &str = "";
        const IGNORED: &str = "________";

        writeln!(f, "ROWS")?;
        writeln!(f, "{EMPTY:1}{:3}{OBJ}", "N")?;
        for constr in &self.constrs {
            if let Some(desc) = &constr.desc {
                writeln!(f, "* {desc}")?
            }
            let typ = match &constr.typ {
                ConstrType::Equal => "E",
                ConstrType::LessEqual => "L",
                ConstrType::GraterEqual => "G",
            };
            writeln!(f, "{EMPTY:1}{typ:3}{}", constr.name)?
        }

        writeln!(f, "COLUMNS")?;
        for var in &self.vars {
            if let Some(desc) = &var.desc {
                writeln!(f, "* {desc}")?
            }
            if let Some(coe) = var.obj_nonzero {
                writeln!(f, "{EMPTY:4}{:10}{OBJ:10}{coe}", var.name)?
            }
            for &(constr_index, coe) in &var.nonzeros {
                let constr = &self.constrs[constr_index];
                writeln!(f, "{EMPTY:4}{:10}{:10}{coe}", var.name, constr.name)?
            }
        }

        writeln!(f, "RHS")?;
        for constr in &self.constrs {
            writeln!(f, "{EMPTY:4}{IGNORED:10}{:10}{}", constr.name, constr.rhs)?
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
            obj: Default::default(),
        }
    }

    pub fn add_var(&mut self, name: String, desc: Option<String>) -> anyhow::Result<usize> {
        anyhow::ensure!(name.len() == 8);
        let index = self.vars.len();
        self.vars.push(Var {
            name,
            desc,
            nonzeros: Default::default(),
            obj_nonzero: None,
        });
        Ok(index)
    }

    pub fn add_constr(&mut self, constr: Constr) -> anyhow::Result<()> {
        anyhow::ensure!(constr.name.len() == 8);
        let index = self.constrs.len();
        for &(coe, var_index) in &constr.cols.0 {
            self.vars[var_index].nonzeros.push((index, coe))
        }
        self.constrs.push(constr);
        Ok(())
    }

    pub fn set_obj(&mut self, objective: Cols) {
        for &(coe, var_index) in &objective.0 {
            self.vars[var_index].obj_nonzero = Some(coe);
        }
        self.obj = objective
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
