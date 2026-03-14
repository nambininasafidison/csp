use std::collections::HashMap;

const LETTERS: [char; 8] = ['I', 'S', 'A', 'R', 'O', 'T', 'E', 'L'];

fn main() {
    let mut assignment = HashMap::new();
    let mut used = [false; 10];

    solve(0, &mut assignment, &mut used);
}

fn word_value(word: &str, map: &HashMap<char, u8>) -> u32 {
    word.chars()
        .fold(0, |acc, c| acc * 10 + *map.get(&c).unwrap() as u32)
}

fn valid_solution(map: &HashMap<char, u8>) -> bool {
    let isa = word_value("ISA", map);
    let roa = word_value("ROA", map);
    let telo = word_value("TELO", map);

    isa + roa == telo
}

fn solve(index: usize, assignment: &mut HashMap<char, u8>, used: &mut [bool; 10]) {
    if index == LETTERS.len() {
        if valid_solution(assignment) {
            println!("Solution trouvée:");
            println!("{:?}", assignment);
        }
        return;
    }

    let letter = LETTERS[index];

    for digit in 0..10 {
        if used[digit] {
            continue;
        }

        if digit == 0 && (letter == 'I' || letter == 'R' || letter == 'T') {
            continue;
        }

        assignment.insert(letter, digit as u8);
        used[digit] = true;

        solve(index + 1, assignment, used);

        used[digit] = false;
        assignment.remove(&letter);
    }
}
