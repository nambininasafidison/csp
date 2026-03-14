use std::collections::HashMap;

fn is_valid(
    assignment: &HashMap<&str, &str>,
    var: &str,
    value: &str,
    neighbors: &HashMap<&str, Vec<&str>>,
) -> bool {
    if let Some(neigh_list) = neighbors.get(var) {
        for neighbor in neigh_list {
            if let Some(&assigned_color) = assignment.get(neighbor) {
                if assigned_color == value {
                    return false;
                }
            }
        }
    }
    true
}

fn backtrack<'a>(
    assignment: &mut HashMap<&'a str, &'a str>,
    variables: &[&'a str],
    colors: &[&'a str],
    neighbors: &HashMap<&'a str, Vec<&'a str>>,
) -> bool {
    if assignment.len() == variables.len() {
        return true;
    }
    let var = variables
        .iter()
        .find(|&&v| !assignment.contains_key(v))
        .copied()
        .unwrap();
    for &color in colors {
        if is_valid(assignment, var, color, neighbors) {
            assignment.insert(var, color);
            if backtrack(assignment, variables, colors, neighbors) {
                return true;
            }
            assignment.remove(var);
        }
    }
    false
}

fn find_color<'a>(
    variables: &[&'a str],
    colors: &[&'a str],
    neighbors: &HashMap<&'a str, Vec<&'a str>>,
) -> Option<HashMap<&'a str, &'a str>> {
    let mut assignment = HashMap::new();
    if backtrack(&mut assignment, variables, colors, neighbors) {
        Some(assignment)
    } else {
        None
    }
}

fn main() {
    let variables = ["A", "B", "C", "D", "E", "F"];
    let colors = ["R", "V", "B"];

    let neighbors: HashMap<&str, Vec<&str>> = [
        ("A", vec!["B", "E"]),
        ("B", vec!["A", "C", "D", "E"]),
        ("C", vec!["B", "D", "E", "F"]),
        ("D", vec!["B", "C", "F"]),
        ("E", vec!["A", "B", "C", "F"]),
        ("F", vec!["C", "D", "E"]),
    ]
    .iter()
    .cloned()
    .collect();

    let city_names: HashMap<&str, &str> = [
        ("A", "Antsiranana"),
        ("B", "Mahajanga"),
        ("C", "Antananarivo"),
        ("D", "Toliara"),
        ("E", "Toamasina"),
        ("F", "Fianarantsoa"),
    ]
    .iter()
    .cloned()
    .collect();

    match find_color(&variables, &colors, &neighbors) {
        Some(mapping) => {
            for code in &variables {
                println!("{} ({}): {}", city_names[code], code, mapping[code]);
            }
        }
        None => println!("Aucun coloriage valide trouve."),
    }
}
