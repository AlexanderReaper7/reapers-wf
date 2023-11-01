#![allow(dead_code)]
use std::fmt::Display;

/// Inserts a space before each capital letter except the first
pub fn separate_camel_case(string: &str) -> String {
    string.chars().enumerate().map(|(i, c)| {
        if i > 0 && c.is_uppercase() {
            format!(" {}", c)
        } else {
            format!("{}", c)
        }
    }).collect::<String>()
}

/// Returns a pretty-printed horizontally aligned string table of the given data
pub fn table(data: Vec<Vec<String>>) -> String {
    let mut max_widths = vec![0; data[0].len()];
    for row in &data {
        for (i, cell) in row.iter().enumerate() {
            if cell.len() > max_widths[i] {
                max_widths[i] = cell.len()
            }
        }
    }
    let mut table = String::new();
    for row in data {
        for (i, cell) in row.iter().enumerate() {
            table.push_str(&format!("{:width$} | ", cell, width = max_widths[i]))
        }
        table.push('\n')
    }
    table
}

pub fn comma_separated_string<T>(input: &Vec<T>) -> String where T: Display {
    input.iter().map(|tier| tier.to_string()).collect::<Vec<String>>().join(", ")
}
