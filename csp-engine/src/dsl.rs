use crate::csp::*;

pub struct CSPModel {
    pub name: String,
    problem: CSPProblem,
}

impl CSPModel {
    pub fn new(name: &str) -> Self {
        CSPModel {
            name: name.to_string(),
            problem: CSPProblem::new(),
        }
    }

    pub fn int_var(mut self, name: &str, range: std::ops::RangeInclusive<i64>) -> Self {
        let domain: Domain = range.collect();
        self.problem.add_variable(name, domain);
        self
    }

    pub fn int_var_domain(mut self, name: &str, domain: Vec<i64>) -> Self {
        self.problem.add_variable(name, domain);
        self
    }

    pub fn int_vars(mut self, names: &[&str], range: std::ops::RangeInclusive<i64>) -> Self {
        let domain: Domain = range.collect();
        for name in names {
            self.problem.add_variable(name, domain.clone());
        }
        self
    }

    pub fn constraint(
        mut self,
        name: &str,
        vars: &[&str],
        check: impl Fn(&Assignment) -> bool + Send + Sync + 'static,
    ) -> Self {
        self.problem.add_constraint(vars.to_vec(), name, check);
        self
    }

    pub fn all_different(mut self, vars: &[&str]) -> Self {
        self.problem.constraints.push(all_different(vars));
        self
    }

    pub fn pairwise_neq(mut self, vars: &[&str]) -> Self {
        for i in 0..vars.len() {
            for j in (i + 1)..vars.len() {
                let vi = vars[i].to_string();
                let vj = vars[j].to_string();
                let name = format!("{} ≠ {}", vi, vj);
                self.problem.constraints.push(Constraint {
                    vars: vec![vi.clone(), vj.clone()],
                    name,
                    check: Box::new(move |a: &Assignment| match (a.get(&vi), a.get(&vj)) {
                        (Some(&a_val), Some(&b_val)) => a_val != b_val,
                        _ => true,
                    }),
                });
            }
        }
        self
    }

    pub fn solve_first(mut self) -> Option<Assignment> {
        println!("model: {}", self.name);
        println!("variables: {}", self.problem.variables.len());
        println!("constraints: {}", self.problem.constraints.len());

        let ac3_ok = self.problem.ac3();
        if !ac3_ok {
            println!("ac3: failed (empty domain, no solution)");
            return None;
        }
        let mut singleton_domains = 0usize;
        let mut min_domain = usize::MAX;
        let mut max_domain = 0usize;
        for v in &self.problem.variables {
            let size = self.problem.domains[v].len();
            if size == 1 {
                singleton_domains += 1;
            }
            min_domain = min_domain.min(size);
            max_domain = max_domain.max(size);
        }
        println!(
            "ac3: ok (singletons={} min_domain={} max_domain={})",
            singleton_domains, min_domain, max_domain
        );

        let (solutions, stats) = solve_mrv(&self.problem, false);
        println!("{}", stats);
        solutions.into_iter().next()
    }

    pub fn solve_all(mut self) -> Vec<Assignment> {
        println!("model: {}", self.name);
        println!("variables: {}", self.problem.variables.len());
        println!("constraints: {}", self.problem.constraints.len());
        println!("solver: mrv + forward-checking + ac3");

        self.problem.ac3();

        let (solutions, stats) = solve_mrv(&self.problem, true);
        println!("{}", stats);
        solutions
    }
}

pub fn isa_roa_telo() -> CSPModel {
    let letters = ["I", "S", "A", "R", "O", "T", "E", "L"];

    let mut model = CSPModel::new("ISA + ROA = TELO");

    for &l in &letters {
        let domain: Domain = if l == "I" || l == "R" || l == "T" {
            (1..=9).collect()
        } else {
            (0..=9).collect()
        };
        model = model.int_var_domain(l, domain);
    }

    model = model.all_different(&letters);

    model = model.constraint("ISA + ROA = TELO", &letters, move |a| {
        let vars_assigned = letters.iter().all(|&l| a.contains_key(l));
        if !vars_assigned {
            return true;
        }
        let isa = a["I"] * 100 + a["S"] * 10 + a["A"];
        let roa = a["R"] * 100 + a["O"] * 10 + a["A"];
        let telo = a["T"] * 1000 + a["E"] * 100 + a["L"] * 10 + a["O"];
        isa + roa == telo
    });

    model
}

pub fn n_queens(n: i64) -> CSPModel {
    let var_names: Vec<String> = (0..n).map(|i| format!("q{}", i)).collect();
    let var_refs: Vec<&str> = var_names.iter().map(|s| s.as_str()).collect();

    let mut model = CSPModel::new(&format!("{}-Queens", n));
    model = model.int_vars(&var_refs, 1..=n);

    model = model.all_different(&var_refs);

    for i in 0..n {
        for j in (i + 1)..n {
            let vi = format!("q{}", i);
            let vj = format!("q{}", j);
            let diff = j - i;
            let name = format!("no_diag({},{})", i, j);

            model.problem.constraints.push(Constraint {
                vars: vec![vi.clone(), vj.clone()],
                name,
                check: Box::new(move |a: &Assignment| match (a.get(&vi), a.get(&vj)) {
                    (Some(&qi), Some(&qj)) => (qi - qj).abs() != diff,
                    _ => true,
                }),
            });
        }
    }

    model
}

pub fn sudoku(grid: [[i64; 9]; 9]) -> CSPModel {
    let mut model = CSPModel::new("Sudoku");

    for (r, row) in grid.iter().enumerate() {
        for (c, cell) in row.iter().enumerate() {
            let name = format!("c{}_{}", r, c);
            if *cell != 0 {
                model = model.int_var_domain(&name, vec![*cell]);
            } else {
                model = model.int_var(&name, 1..=9);
            }
        }
    }

    for r in 0..9 {
        let vars: Vec<String> = (0..9).map(|c| format!("c{}_{}", r, c)).collect();
        let refs: Vec<&str> = vars.iter().map(|s| s.as_str()).collect();
        model = model.pairwise_neq(&refs);
    }

    for c in 0..9 {
        let vars: Vec<String> = (0..9).map(|r| format!("c{}_{}", r, c)).collect();
        let refs: Vec<&str> = vars.iter().map(|s| s.as_str()).collect();
        model = model.pairwise_neq(&refs);
    }

    for br in 0..3 {
        for bc in 0..3 {
            let vars: Vec<String> = (0..3)
                .flat_map(|r| (0..3).map(move |c| format!("c{}_{}", br * 3 + r, bc * 3 + c)))
                .collect();
            let refs: Vec<&str> = vars.iter().map(|s| s.as_str()).collect();
            model = model.pairwise_neq(&refs);
        }
    }

    model
}

pub fn graph_coloring(regions: &[&str], edges: &[(&str, &str)], n_colors: i64) -> CSPModel {
    let mut model = CSPModel::new("Graph Coloring");

    for &region in regions {
        model = model.int_var(region, 1..=n_colors);
    }

    for &(a, b) in edges {
        let va = a.to_string();
        let vb = b.to_string();
        let name = format!("{} ≠ {}", a, b);
        model.problem.constraints.push(Constraint {
            vars: vec![va.clone(), vb.clone()],
            name,
            check: Box::new(move |assignment: &Assignment| {
                match (assignment.get(&va), assignment.get(&vb)) {
                    (Some(&a_val), Some(&b_val)) => a_val != b_val,
                    _ => true,
                }
            }),
        });
    }

    model
}
