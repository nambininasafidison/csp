mod cryptarithm;
mod csp;
mod dsl;

use std::time::Instant;

fn main() {
    println!("csp-engine: rust constraint solver");
    println!("features: ac3, mrv, forward-checking, rayon");

    println!("benchmark: ISA + ROA = TELO");
    {
        let problem = build_isa_problem();

        println!("\nstep 1/3: naive backtracking");
        let t = Instant::now();
        let (solutions, stats) = csp::solve_naive(&problem, true);
        let naive_time = t.elapsed();
        println!("{}", stats);
        println!("time={:?} solutions={}", naive_time, solutions.len());
        for sol in solutions.iter().take(3) {
            display_isa_solution(sol);
        }
        if solutions.len() > 3 {
            println!("... {} more solution(s) not displayed", solutions.len() - 3);
        }

        let problem = build_isa_problem();
        println!("\nstep 2/3: mrv + forward-checking");
        let t = Instant::now();
        let (solutions, stats) = csp::solve_mrv(&problem, true);
        let mrv_time = t.elapsed();
        println!("{}", stats);
        println!("time={:?} solutions={}", mrv_time, solutions.len());

        let problem = build_isa_problem();
        println!("\nstep 3/3: parallel rayon");
        let t = Instant::now();
        let (solutions, stats) = csp::solve_parallel(&problem, true);
        let par_time = t.elapsed();
        println!("{}", stats);
        println!("time={:?} solutions={}", par_time, solutions.len());

        if mrv_time.as_nanos() > 0 {
            let speedup = naive_time.as_nanos() as f64 / mrv_time.as_nanos() as f64;
            println!("speedup mrv_vs_naive={:.1}x", speedup);
        }
        if par_time.as_nanos() > 0 {
            let speedup = naive_time.as_nanos() as f64 / par_time.as_nanos() as f64;
            println!("speedup parallel_vs_naive={:.1}x", speedup);
        }
    }

    println!("\ncryptarithms: generic solver");

    let puzzles = ["ISA + ROA = TELO", "AB + BA = CBC", "NO + GUN = HUNT"];

    for puzzle in &puzzles {
        println!("\npuzzle: {}", puzzle);
        let t = Instant::now();
        let (solutions, stats) = cryptarithm::solve_cryptarithm(puzzle, "mrv", true);
        println!(
            "{} time={:?} solutions={}",
            stats,
            t.elapsed(),
            solutions.len()
        );
    }

    println!("\ngraph coloring: madagascar");
    {
        let regions = [
            "Antsiranana",
            "Mahajanga",
            "Antananarivo",
            "Toliara",
            "Toamasina",
            "Fianarantsoa",
        ];
        let edges = [
            ("Antsiranana", "Mahajanga"),
            ("Antsiranana", "Toamasina"),
            ("Mahajanga", "Antananarivo"),
            ("Mahajanga", "Toamasina"),
            ("Mahajanga", "Toliara"),
            ("Antananarivo", "Toamasina"),
            ("Antananarivo", "Fianarantsoa"),
            ("Antananarivo", "Toliara"),
            ("Toamasina", "Fianarantsoa"),
            ("Toliara", "Fianarantsoa"),
        ];
        let color_names = ["Rouge", "Vert", "Bleu"];
        let t = Instant::now();
        let model = dsl::graph_coloring(&regions, &edges, 3);
        if let Some(solution) = model.solve_first() {
            for region in &regions {
                let color_idx = solution[*region] as usize - 1;
                println!("region={} color={}", region, color_names[color_idx]);
            }
        }
        println!("time={:?}", t.elapsed());
    }

    println!("\ndsl: ISA + ROA = TELO (all solutions)");
    {
        let t = Instant::now();
        let model = dsl::isa_roa_telo();
        let solutions = model.solve_all();
        println!("time={:?} solutions={}", t.elapsed(), solutions.len());
        for (i, sol) in solutions.iter().take(5).enumerate() {
            let isa = sol["I"] * 100 + sol["S"] * 10 + sol["A"];
            let roa = sol["R"] * 100 + sol["O"] * 10 + sol["A"];
            let telo = sol["T"] * 1000 + sol["E"] * 100 + sol["L"] * 10 + sol["O"];
            println!("solution #{}: {} + {} = {}", i + 1, isa, roa, telo);
        }
        if solutions.len() > 5 {
            println!("... {} more solution(s) not displayed", solutions.len() - 5);
        }
    }

    println!("\nn-queens: mrv + forward-checking + ac3");
    {
        for n in [4, 8] {
            let t = Instant::now();
            let model = dsl::n_queens(n);
            let solution = model.solve_first();
            let elapsed = t.elapsed();
            match solution {
                Some(sol) => {
                    print!("{}-queens time={:?} positions=[", n, elapsed);
                    for i in 0..n {
                        let key = format!("q{}", i);
                        print!("{}", sol[&key]);
                        if i < n - 1 {
                            print!(", ");
                        }
                    }
                    println!("]");
                    if n == 8 {
                        println!();
                        for row in 1..=n {
                            print!("    ");
                            for col in 0..n {
                                let key = format!("q{}", col);
                                if sol[&key] == row {
                                    print!("Q ");
                                } else {
                                    print!(". ");
                                }
                            }
                            println!();
                        }
                    }
                }
                None => println!("{}-queens: no solution", n),
            }
        }
    }

    println!("\nsudoku: ac3 + mrv");
    {
        let grid = [
            [5, 3, 0, 0, 7, 0, 0, 0, 0],
            [6, 0, 0, 1, 9, 5, 0, 0, 0],
            [0, 9, 8, 0, 0, 0, 0, 6, 0],
            [8, 0, 0, 0, 6, 0, 0, 0, 3],
            [4, 0, 0, 8, 0, 3, 0, 0, 1],
            [7, 0, 0, 0, 2, 0, 0, 0, 6],
            [0, 6, 0, 0, 0, 0, 2, 8, 0],
            [0, 0, 0, 4, 1, 9, 0, 0, 5],
            [0, 0, 0, 0, 8, 0, 0, 7, 9],
        ];

        let t = Instant::now();
        let model = dsl::sudoku(grid);
        let solution = model.solve_first();
        let elapsed = t.elapsed();

        match solution {
            Some(sol) => {
                println!("solved in {:?}", elapsed);
                for r in 0..9 {
                    for c in 0..9 {
                        let key = format!("c{}_{}", r, c);
                        print!("{} ", sol[&key]);
                    }
                    println!();
                }
            }
            None => println!("no solution found"),
        }
    }

    println!("\nall demos completed successfully");
}

fn build_isa_problem() -> csp::CSPProblem {
    let mut problem = csp::CSPProblem::new();
    let letters = ["I", "S", "A", "R", "O", "T", "E", "L"];

    for &l in &letters {
        let domain: csp::Domain = if l == "I" || l == "R" || l == "T" {
            (1..=9).collect()
        } else {
            (0..=9).collect()
        };
        problem.add_variable(l, domain);
    }

    problem.constraints.push(csp::all_different(&letters));

    problem.add_constraint(letters.to_vec(), "ISA + ROA = TELO", |a| {
        let keys = ["I", "S", "A", "R", "O", "T", "E", "L"];
        if !keys.iter().all(|&k| a.contains_key(k)) {
            return true;
        }
        let isa = a["I"] * 100 + a["S"] * 10 + a["A"];
        let roa = a["R"] * 100 + a["O"] * 10 + a["A"];
        let telo = a["T"] * 1000 + a["E"] * 100 + a["L"] * 10 + a["O"];
        isa + roa == telo
    });

    problem
}

fn display_isa_solution(sol: &csp::Assignment) {
    let isa = sol["I"] * 100 + sol["S"] * 10 + sol["A"];
    let roa = sol["R"] * 100 + sol["O"] * 10 + sol["A"];
    let telo = sol["T"] * 1000 + sol["E"] * 100 + sol["L"] * 10 + sol["O"];
    println!("solution: {} + {} = {}", isa, roa, telo);
}
