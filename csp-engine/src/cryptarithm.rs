use crate::csp::*;
use std::collections::HashSet;

#[derive(Debug)]
pub struct Cryptarithm {
    pub words_left: Vec<String>,
    pub word_right: String,
    pub letters: Vec<char>,
    pub leading_chars: HashSet<char>,
}

impl Cryptarithm {
    pub fn parse(expr: &str) -> Self {
        let parts: Vec<&str> = expr.split('=').collect();
        assert!(parts.len() == 2, "Expression must contain exactly one '='");

        let left = parts[0].trim();
        let right = parts[1].trim();

        let words_left: Vec<String> = left.split('+').map(|w| w.trim().to_uppercase()).collect();
        let word_right = right.to_uppercase();

        let mut letter_set = HashSet::new();
        let mut leading_chars = HashSet::new();

        for word in &words_left {
            for c in word.chars() {
                letter_set.insert(c);
            }
            if let Some(c) = word.chars().next() {
                leading_chars.insert(c);
            }
        }
        for c in word_right.chars() {
            letter_set.insert(c);
        }
        if let Some(c) = word_right.chars().next() {
            leading_chars.insert(c);
        }

        let mut letters: Vec<char> = letter_set.into_iter().collect();
        letters.sort();

        Cryptarithm {
            words_left,
            word_right,
            letters,
            leading_chars,
        }
    }

    pub fn to_csp(&self) -> CSPProblem {
        let mut problem = CSPProblem::new();

        for &letter in &self.letters {
            let domain: Domain = if self.leading_chars.contains(&letter) {
                (1..=9).collect()
            } else {
                (0..=9).collect()
            };
            problem.add_variable(&letter.to_string(), domain);
        }

        let letters_clone = self.letters.clone();
        problem.constraints.push(Constraint {
            vars: self.letters.iter().map(|c| c.to_string()).collect(),
            name: "AllDifferent".to_string(),
            check: Box::new(move |assignment: &Assignment| {
                let mut seen = HashSet::new();
                for c in &letters_clone {
                    if let Some(&val) = assignment.get(&c.to_string())
                        && !seen.insert(val)
                    {
                        return false;
                    }
                }
                true
            }),
        });

        let words_left = self.words_left.clone();
        let word_right = self.word_right.clone();
        let all_vars: Vec<String> = self.letters.iter().map(|c| c.to_string()).collect();

        problem.constraints.push(Constraint {
            vars: all_vars,
            name: format!("{} = {}", words_left.join(" + "), word_right),
            check: Box::new(move |assignment: &Assignment| {
                let mut left_sum: i64 = 0;
                for word in &words_left {
                    let mut val: i64 = 0;
                    for c in word.chars() {
                        if let Some(&d) = assignment.get(&c.to_string()) {
                            val = val * 10 + d;
                        } else {
                            return true;
                        }
                    }
                    left_sum += val;
                }

                let mut right_val: i64 = 0;
                for c in word_right.chars() {
                    if let Some(&d) = assignment.get(&c.to_string()) {
                        right_val = right_val * 10 + d;
                    } else {
                        return true;
                    }
                }

                left_sum == right_val
            }),
        });

        problem
    }

    pub fn display_solution(&self, assignment: &Assignment) {
        fn word_val(word: &str, a: &Assignment) -> i64 {
            word.chars().fold(0i64, |acc, c| {
                acc * 10 + a.get(&c.to_string()).copied().unwrap_or(0)
            })
        }

        let left_vals: Vec<String> = self
            .words_left
            .iter()
            .map(|w| format!("{} ({})", w, word_val(w, assignment)))
            .collect();
        let right_val = word_val(&self.word_right, assignment);

        println!(
            "    equation: {} = {} ({})",
            left_vals.join(" + "),
            self.word_right,
            right_val
        );
        println!(
            "    mapping: {}",
            self.letters
                .iter()
                .map(|c| format!("{}={}", c, assignment.get(&c.to_string()).unwrap()))
                .collect::<Vec<_>>()
                .join("  ")
        );
    }
}

pub fn solve_cryptarithm(
    expr: &str,
    solver: &str,
    find_all: bool,
) -> (Vec<Assignment>, SolveStats) {
    let crypto = Cryptarithm::parse(expr);
    let problem = crypto.to_csp();

    let result = match solver {
        "naive" => solve_naive(&problem, find_all),
        "mrv" => solve_mrv(&problem, find_all),
        "parallel" => solve_parallel(&problem, find_all),
        _ => solve_mrv(&problem, find_all),
    };

    if !result.0.is_empty() {
        let preview = result.0.len().min(3);
        for (i, sol) in result.0.iter().take(preview).enumerate() {
            println!("    solution #{}", i + 1);
            crypto.display_solution(sol);
        }
        if result.0.len() > preview {
            println!(
                "    ... {} more solution(s) not displayed",
                result.0.len() - preview
            );
        }
    }

    result
}
