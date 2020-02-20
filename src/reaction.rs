use crate::chemical::*;

pub type Result<T> = std::result::Result<T, ReactionError>;

use std::collections::HashSet;

#[derive(Debug)]
pub enum ReactionError {
    UnbalancedElements,
    InfiniteSolution,
}

pub fn calculate_coefficients(reagents: &[Chemical], products: &[Chemical]) -> Result<Vec<i64>> {
    let linear_system = create_linear_equation(reagents, products)?;
    integer_gauss(linear_system)
}

fn get_elements_involved(reagents: &[Chemical], products: &[Chemical]) -> Result<Vec<String>> {
    let mut element_list = HashSet::new();
    for reagent in reagents {
        for element in reagent.parts.keys() {
            element_list.insert(element.clone());
        }
    }
    for product in products {
        let mut elements = product.parts.keys();
        if elements.any(|element| !element_list.contains(element)) {
            return Err(ReactionError::UnbalancedElements);
        }
    }
    Ok(element_list.iter().cloned().collect())
}

struct ReactionMatrix {
    matrix: Vec<i64>,
    columns: usize,
}

fn create_linear_equation(reagents: &[Chemical], products: &[Chemical]) -> Result<ReactionMatrix> {
    let elements_involved = get_elements_involved(reagents, products)?;
    let columns = reagents.len() + products.len();
    let mut matrix = Vec::new();

    for element in &elements_involved {
        for reagent in reagents {
            let coefficient = reagent.parts.get(element).cloned().unwrap_or(0) as i64;
            matrix.push(coefficient);
        }
        for product in products {
            let coefficient = product.parts.get(element).cloned().unwrap_or(0) as i64;
            matrix.push(-coefficient);
        }
    }

    Ok(ReactionMatrix { matrix, columns })
}

fn integer_gauss(matrix: ReactionMatrix) -> Result<Vec<i64>> {
    let ReactionMatrix {
        mut matrix,
        columns,
    } = matrix;

    let rows = matrix.len() / columns;
    let least_required_rows = columns - 1;
    if rows < least_required_rows {
        return Err(ReactionError::InfiniteSolution);
    }

    for row in 0..least_required_rows {
        let first_term_column = row;
        if matrix[row * columns + first_term_column] == 0 {
            for other_row in (row + 1)..rows {
                if matrix[other_row * columns + first_term_column] != 0 {
                    swap_row(&mut matrix, row, other_row, columns);
                    break;
                }
            }
        }

        for other_row in (row + 1)..rows {
            cancel_row(&mut matrix, row, other_row, columns, first_term_column);
        }
    }

    let mut solutions = vec![1];
    for row in (0..least_required_rows).rev() {
        let mut other_sum = 0i64;
        for (solution_index, other_term) in (row + 1..columns).rev().enumerate() {
            let coefficient = matrix[row * columns + other_term];
            let value = solutions[solution_index];
            other_sum += coefficient * value;
        }
        other_sum = other_sum.abs();
        // equation ax + other_sum = 0
        let first_coefficient = matrix[row * columns + row].abs();
        let (solution, other_factor) = {
            let lcm = lcm(first_coefficient, other_sum);
            (lcm / first_coefficient, lcm / other_sum)
        };
        solutions.iter_mut().for_each(|sol| *sol *= other_factor);
        solutions.push(solution);
    }

    solutions.reverse();
    Ok(solutions)
}

fn swap_row<T>(vec: &mut Vec<T>, row1: usize, row2: usize, columns: usize) {
    let row1_start_index = row1 * columns;
    let row2_start_index = row2 * columns;
    for column in 0..columns {
        let index1 = row1_start_index + column;
        let index2 = row2_start_index + column;
        vec.swap(index1, index2);
    }
}

fn cancel_row(vec: &mut Vec<i64>, row1: usize, row2: usize, columns: usize, first_nonzero: usize) {
    let row1_start_index = row1 * columns;
    let row2_start_index = row2 * columns;
    if vec[row2_start_index + first_nonzero] != 0 {
        let (row1_factor, row2_factor) = {
            let lcm = lcm(
                vec[row1_start_index + first_nonzero].abs(),
                vec[row2_start_index + first_nonzero].abs(),
            );
            (
                lcm / vec[row1_start_index + first_nonzero],
                lcm / vec[row2_start_index + first_nonzero],
            )
        };

        for column in first_nonzero..columns {
            let cancelled = vec[row1_start_index + column] * row1_factor
                - vec[row2_start_index + column] * row2_factor;
            vec[row2_start_index + column] = cancelled;
        }
    }
}

fn lcm(a: i64, b: i64) -> i64 {
    assert!(a > 0, "a must be bigger than 0, found: {}", a);
    assert!(b > 0, "b must be bigger than 0, found: {}", b);
    fn gcd(mut a: i64, mut b: i64) -> i64 {
        if a == b {
            a
        } else {
            let mut shift = 0;
            while ((a | b) & 1) == 0 {
                shift += 1;
                a >>= 1;
                b >>= 1;
            }
            while a & 1 == 0 {
                a >>= 1;
            }
            loop {
                while (b & 1) == 0 {
                    b >>= 1;
                }
                if a > b {
                    std::mem::swap(&mut a, &mut b);
                }
                b -= a;
                if b == 0 {
                    break;
                }
            }
            a << shift
        }
    }
    a * b / gcd(a, b)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn calculate() {
        let reagents = vec![parse_chemical("H2O").unwrap()];
        let products = vec![parse_chemical("H2").unwrap(), parse_chemical("O2").unwrap()];
        let solution = calculate_coefficients(&reagents, &products).unwrap();
        assert_eq!(vec![2, 2, 1], solution);
    }

    #[test]
    fn calculate_complicated() {
        let reagents = vec![
            parse_chemical("C15H31COONa").unwrap(),
            parse_chemical("CaCl2").unwrap(),
        ];
        let products = vec![
            parse_chemical("(C15H31COO)2Ca").unwrap(),
            parse_chemical("NaCl").unwrap(),
        ];
        let solution = calculate_coefficients(&reagents, &products).unwrap();
        assert_eq!(vec![2, 1, 1, 2], solution);
    }
}
