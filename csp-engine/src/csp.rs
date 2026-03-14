use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

pub type Domain = Vec<i64>;
pub type Assignment = HashMap<String, i64>;

pub struct Constraint {
    pub vars: Vec<String>,
    pub name: String,
    pub check: Box<dyn Fn(&Assignment) -> bool + Send + Sync>,
}

impl fmt::Debug for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Constraint({}: {:?})", self.name, self.vars)
    }
}

pub struct CSPProblem {
    pub variables: Vec<String>,
    pub domains: HashMap<String, Domain>,
    pub constraints: Vec<Constraint>,
}

impl CSPProblem {
    pub fn new() -> Self {
        CSPProblem {
            variables: Vec::new(),
            domains: HashMap::new(),
            constraints: Vec::new(),
        }
    }

    pub fn add_variable(&mut self, name: &str, domain: Domain) {
        self.variables.push(name.to_string());
        self.domains.insert(name.to_string(), domain);
    }

    pub fn add_constraint(
        &mut self,
        vars: Vec<&str>,
        name: &str,
        check: impl Fn(&Assignment) -> bool + Send + Sync + 'static,
    ) {
        self.constraints.push(Constraint {
            vars: vars.iter().map(|s| s.to_string()).collect(),
            name: name.to_string(),
            check: Box::new(check),
        });
    }

    pub fn constraints_for(&self, var: &str) -> Vec<&Constraint> {
        self.constraints
            .iter()
            .filter(|c| c.vars.iter().any(|v| v == var))
            .collect()
    }

    pub fn is_consistent(&self, assignment: &Assignment) -> bool {
        for constraint in &self.constraints {
            let all_assigned = constraint.vars.iter().all(|v| assignment.contains_key(v));
            if all_assigned && !(constraint.check)(assignment) {
                return false;
            }
        }
        true
    }

    pub fn is_complete(&self, assignment: &Assignment) -> bool {
        self.variables.iter().all(|v| assignment.contains_key(v))
    }

    pub fn ac3(&mut self) -> bool {
        let mut queue: VecDeque<(String, String, usize)> = VecDeque::new();

        for (ci, constraint) in self.constraints.iter().enumerate() {
            for i in 0..constraint.vars.len() {
                for j in 0..constraint.vars.len() {
                    if i != j {
                        queue.push_back((
                            constraint.vars[i].clone(),
                            constraint.vars[j].clone(),
                            ci,
                        ));
                    }
                }
            }
        }

        while let Some((xi, xj, ci)) = queue.pop_front() {
            if self.revise(&xi, &xj, ci) {
                if self.domains[&xi].is_empty() {
                    return false;
                }
                for (ck, constraint) in self.constraints.iter().enumerate() {
                    if constraint.vars.contains(&xi) {
                        for xk in &constraint.vars {
                            if xk != &xi && xk != &xj {
                                queue.push_back((xk.clone(), xi.clone(), ck));
                            }
                        }
                    }
                }
            }
        }
        true
    }

    fn revise(&mut self, xi: &str, xj: &str, ci: usize) -> bool {
        let mut revised = false;
        let xi_domain = self.domains[xi].clone();
        let xj_domain = self.domains[xj].clone();

        let mut new_domain = Vec::new();

        for &vali in &xi_domain {
            let mut satisfiable = false;
            for &valj in &xj_domain {
                let mut assignment = Assignment::new();
                assignment.insert(xi.to_string(), vali);
                assignment.insert(xj.to_string(), valj);

                if (self.constraints[ci].check)(&assignment) {
                    satisfiable = true;
                    break;
                }
            }
            if satisfiable {
                new_domain.push(vali);
            } else {
                revised = true;
            }
        }

        self.domains.insert(xi.to_string(), new_domain);
        revised
    }

    pub fn forward_check(
        &self,
        var: &str,
        _val: i64,
        assignment: &Assignment,
        domains: &mut HashMap<String, Domain>,
    ) -> Option<HashMap<String, Vec<i64>>> {
        let mut pruned: HashMap<String, Vec<i64>> = HashMap::new();

        for constraint in self.constraints_for(var) {
            for neighbor in &constraint.vars {
                if neighbor == var || assignment.contains_key(neighbor) {
                    continue;
                }

                let dom = domains.get(neighbor).cloned().unwrap_or_default();
                let mut new_dom = Vec::new();
                let mut removed = Vec::new();

                for &val in &dom {
                    let mut test_assign = assignment.clone();
                    test_assign.insert(neighbor.clone(), val);

                    let all_assigned = constraint.vars.iter().all(|v| test_assign.contains_key(v));
                    if !all_assigned || (constraint.check)(&test_assign) {
                        new_dom.push(val);
                    } else {
                        removed.push(val);
                    }
                }

                if new_dom.is_empty() {
                    return None;
                }

                if !removed.is_empty() {
                    pruned.entry(neighbor.clone()).or_default().extend(removed);
                    domains.insert(neighbor.clone(), new_dom);
                }
            }
        }

        Some(pruned)
    }

    pub fn undo_pruning(
        &self,
        pruned: &HashMap<String, Vec<i64>>,
        domains: &mut HashMap<String, Domain>,
    ) {
        for (var, values) in pruned {
            if let Some(dom) = domains.get_mut(var) {
                dom.extend(values);
                dom.sort();
                dom.dedup();
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SolveStats {
    pub nodes_explored: u64,
    pub solutions_found: u64,
    pub backtracks: u64,
    pub pruned_values: u64,
}

impl SolveStats {
    pub fn new() -> Self {
        SolveStats {
            nodes_explored: 0,
            solutions_found: 0,
            backtracks: 0,
            pruned_values: 0,
        }
    }
}

impl fmt::Display for SolveStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "nodes={:>8} solutions={:>4} backtracks={:>8} pruned={:>6}",
            self.nodes_explored, self.solutions_found, self.backtracks, self.pruned_values
        )
    }
}

pub fn solve_naive(problem: &CSPProblem, find_all: bool) -> (Vec<Assignment>, SolveStats) {
    let mut stats = SolveStats::new();
    let mut solutions = Vec::new();
    let mut assignment = Assignment::new();

    fn backtrack(
        problem: &CSPProblem,
        assignment: &mut Assignment,
        solutions: &mut Vec<Assignment>,
        stats: &mut SolveStats,
        find_all: bool,
    ) -> bool {
        stats.nodes_explored += 1;

        if problem.is_complete(assignment) {
            if problem.is_consistent(assignment) {
                stats.solutions_found += 1;
                solutions.push(assignment.clone());
                return !find_all;
            }
            return false;
        }
        let var = problem
            .variables
            .iter()
            .find(|v| !assignment.contains_key(*v))
            .unwrap()
            .clone();

        let domain = problem.domains[&var].clone();
        for val in domain {
            assignment.insert(var.clone(), val);

            if problem.is_consistent(assignment)
                && backtrack(problem, assignment, solutions, stats, find_all)
            {
                return true;
            }

            assignment.remove(&var);
            stats.backtracks += 1;
        }

        false
    }

    backtrack(
        problem,
        &mut assignment,
        &mut solutions,
        &mut stats,
        find_all,
    );
    (solutions, stats)
}

pub fn solve_mrv(problem: &CSPProblem, find_all: bool) -> (Vec<Assignment>, SolveStats) {
    let mut stats = SolveStats::new();
    let mut solutions = Vec::new();
    let mut assignment = Assignment::new();
    let mut domains = problem.domains.clone();

    fn select_mrv_variable(
        problem: &CSPProblem,
        assignment: &Assignment,
        domains: &HashMap<String, Domain>,
    ) -> Option<String> {
        let mut best: Option<(String, usize)> = None;

        for var in &problem.variables {
            if assignment.contains_key(var) {
                continue;
            }
            let size = domains.get(var).map(|d| d.len()).unwrap_or(0);
            match &best {
                None => best = Some((var.clone(), size)),
                Some((_, best_size)) => {
                    if size < *best_size {
                        best = Some((var.clone(), size));
                    }
                }
            }
        }

        best.map(|(v, _)| v)
    }

    fn backtrack_mrv(
        problem: &CSPProblem,
        assignment: &mut Assignment,
        domains: &mut HashMap<String, Domain>,
        solutions: &mut Vec<Assignment>,
        stats: &mut SolveStats,
        find_all: bool,
    ) -> bool {
        stats.nodes_explored += 1;

        if problem.is_complete(assignment) {
            stats.solutions_found += 1;
            solutions.push(assignment.clone());
            return !find_all;
        }

        let var = match select_mrv_variable(problem, assignment, domains) {
            Some(v) => v,
            None => return false,
        };

        let domain = domains[&var].clone();
        for val in domain {
            assignment.insert(var.clone(), val);

            if problem.is_consistent(assignment)
                && let Some(pruned) = problem.forward_check(&var, val, assignment, domains)
            {
                stats.pruned_values += pruned.values().map(|v| v.len() as u64).sum::<u64>();

                if backtrack_mrv(problem, assignment, domains, solutions, stats, find_all) {
                    problem.undo_pruning(&pruned, domains);
                    return true;
                }

                problem.undo_pruning(&pruned, domains);
            }

            assignment.remove(&var);
            stats.backtracks += 1;
        }

        false
    }

    backtrack_mrv(
        problem,
        &mut assignment,
        &mut domains,
        &mut solutions,
        &mut stats,
        find_all,
    );
    (solutions, stats)
}

pub fn solve_parallel(problem: &CSPProblem, find_all: bool) -> (Vec<Assignment>, SolveStats) {
    use rayon::prelude::*;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicU64, Ordering};

    let first_var = problem.variables[0].clone();
    let first_domain = problem.domains[&first_var].clone();

    let solutions = Mutex::new(Vec::new());
    let total_nodes = AtomicU64::new(0);
    let total_backtracks = AtomicU64::new(0);
    let total_pruned = AtomicU64::new(0);
    let total_solutions = AtomicU64::new(0);

    first_domain.par_iter().for_each(|&val| {
        let mut assignment = Assignment::new();
        assignment.insert(first_var.clone(), val);

        if !problem.is_consistent(&assignment) {
            total_backtracks.fetch_add(1, Ordering::Relaxed);
            return;
        }

        let mut domains = problem.domains.clone();
        domains.insert(first_var.clone(), vec![val]);

        let mut local_solutions = Vec::new();
        let mut stats = SolveStats::new();

        parallel_backtrack(
            problem,
            &mut assignment,
            &mut domains,
            &mut local_solutions,
            &mut stats,
            find_all,
            1,
        );

        total_nodes.fetch_add(stats.nodes_explored, Ordering::Relaxed);
        total_backtracks.fetch_add(stats.backtracks, Ordering::Relaxed);
        total_pruned.fetch_add(stats.pruned_values, Ordering::Relaxed);
        total_solutions.fetch_add(stats.solutions_found, Ordering::Relaxed);

        if !local_solutions.is_empty() {
            let mut sols = solutions.lock().unwrap();
            sols.extend(local_solutions);
        }
    });

    let stats = SolveStats {
        nodes_explored: total_nodes.load(Ordering::Relaxed),
        solutions_found: total_solutions.load(Ordering::Relaxed),
        backtracks: total_backtracks.load(Ordering::Relaxed),
        pruned_values: total_pruned.load(Ordering::Relaxed),
    };

    (solutions.into_inner().unwrap(), stats)
}

fn parallel_backtrack(
    problem: &CSPProblem,
    assignment: &mut Assignment,
    domains: &mut HashMap<String, Domain>,
    solutions: &mut Vec<Assignment>,
    stats: &mut SolveStats,
    find_all: bool,
    _depth: usize,
) -> bool {
    stats.nodes_explored += 1;

    if problem.is_complete(assignment) {
        stats.solutions_found += 1;
        solutions.push(assignment.clone());
        return !find_all;
    }

    let var = {
        let mut best: Option<(String, usize)> = None;
        for v in &problem.variables {
            if assignment.contains_key(v) {
                continue;
            }
            let size = domains.get(v).map(|d| d.len()).unwrap_or(0);
            if size == 0 {
                return false;
            }
            match &best {
                None => best = Some((v.clone(), size)),
                Some((_, bs)) if size < *bs => best = Some((v.clone(), size)),
                _ => {}
            }
        }
        match best {
            Some((v, _)) => v,
            None => return false,
        }
    };

    let domain = domains[&var].clone();
    for val in domain {
        assignment.insert(var.clone(), val);

        if problem.is_consistent(assignment)
            && let Some(pruned) = problem.forward_check(&var, val, assignment, domains)
        {
            stats.pruned_values += pruned.values().map(|v| v.len() as u64).sum::<u64>();

            if parallel_backtrack(
                problem,
                assignment,
                domains,
                solutions,
                stats,
                find_all,
                _depth + 1,
            ) {
                problem.undo_pruning(&pruned, domains);
                return true;
            }
            problem.undo_pruning(&pruned, domains);
        }

        assignment.remove(&var);
        stats.backtracks += 1;
    }

    false
}

pub fn all_different(vars: &[&str]) -> Constraint {
    let var_names: Vec<String> = vars.iter().map(|s| s.to_string()).collect();
    let vars_clone = var_names.clone();
    Constraint {
        vars: var_names,
        name: "AllDifferent".to_string(),
        check: Box::new(move |assignment: &Assignment| {
            let mut seen = HashSet::new();
            for v in &vars_clone {
                if let Some(&val) = assignment.get(v)
                    && !seen.insert(val)
                {
                    return false;
                }
            }
            true
        }),
    }
}
