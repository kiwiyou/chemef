use std::collections::HashMap;

#[derive(Debug)]
pub struct Chemical {
    pub parts: HashMap<String, usize>,
    pub display: String,
}

pub fn parse_chemical(input: impl AsRef<str>) -> Option<Chemical> {
    enum State {
        None,
        ShallowLetter,
        ShallowDigit,
        DeepNone,
        DeepLetter,
        DeepDigit,
        DeepEnd,
        CompositeDigit,
    }

    let mut name = String::new();
    let mut count = 0usize;
    let mut composite_count = 0usize;
    let mut parts = HashMap::new();
    let mut parts_stack = Vec::new();
    let mut state = State::None;
    let input = input.as_ref();

    for c in input.chars() {
        match (state, c) {
            (State::None, 'A'..='Z') => {
                name.push(c);
                state = State::ShallowLetter;
            }
            (State::None, '(') => {
                parts_stack.push(parts);
                parts = HashMap::new();
                state = State::DeepNone;
            }
            (State::ShallowLetter, 'A'..='Z') => {
                create_or_add(&mut parts, name, 1);
                name = String::new();
                name.push(c);
                state = State::ShallowLetter;
            }
            (State::ShallowLetter, 'a'..='z') => {
                name.push(c);
                state = State::ShallowLetter;
            }
            (State::ShallowLetter, '1'..='9') => {
                count = c as usize - '0' as usize;
                state = State::ShallowDigit;
            }
            (State::ShallowLetter, '(') | (State::DeepLetter, '(') => {
                create_or_add(&mut parts, name, 1);
                name = String::new();
                parts_stack.push(parts);
                parts = HashMap::new();
                state = State::DeepNone;
            }
            (State::ShallowDigit, 'A'..='Z') => {
                create_or_add(&mut parts, name, count);
                name = String::new();
                name.push(c);
                state = State::ShallowLetter;
            }
            (State::ShallowDigit, '0'..='9') => {
                count = count * 10 + (c as usize - '0' as usize);
                state = State::ShallowDigit;
            }
            (State::ShallowDigit, '(') | (State::DeepDigit, '(') => {
                create_or_add(&mut parts, name, count);
                name = String::new();
                parts_stack.push(parts);
                parts = HashMap::new();
                state = State::DeepNone;
            }
            (State::DeepNone, 'A'..='Z') => {
                name.push(c);
                state = State::DeepLetter;
            }
            (State::DeepLetter, 'A'..='Z') => {
                create_or_add(&mut parts, name, 1);
                name = String::new();
                name.push(c);
                state = State::DeepLetter;
            }
            (State::DeepLetter, 'a'..='z') => {
                name.push(c);
                state = State::DeepLetter;
            }
            (State::DeepLetter, '1'..='9') => {
                count = c as usize - '0' as usize;
                state = State::DeepDigit;
            }
            (State::DeepLetter, ')') => {
                create_or_add(&mut parts, name, 1);
                name = String::new();
                state = State::DeepEnd;
            }
            (State::DeepDigit, 'A'..='Z') => {
                create_or_add(&mut parts, name, count);
                name = String::new();
                name.push(c);
                state = State::DeepLetter;
            }
            (State::DeepDigit, '0'..='9') => {
                count = count * 10 + (c as usize - '0' as usize);
                state = State::DeepDigit;
            }
            (State::DeepDigit, ')') => {
                create_or_add(&mut parts, name, count);
                name = String::new();
                state = State::DeepEnd;
            }
            (State::DeepEnd, '1'..='9') => {
                composite_count = c as usize - '0' as usize;
                state = State::CompositeDigit;
            }
            (State::DeepEnd, _) => {
                let mut saved_parts = parts_stack
                    .pop()
                    .expect("State::DeepEnd with empty saved_parts");
                for (name, count) in parts.iter() {
                    if let Some(saved_count) = saved_parts.get_mut(name) {
                        *saved_count += count;
                    } else {
                        saved_parts.insert(name.clone(), *count);
                    }
                }
                parts = saved_parts;
                match c {
                    'A'..='Z' => {
                        name.push(c);
                        if parts_stack.is_empty() {
                            state = State::ShallowLetter;
                        } else {
                            state = State::DeepLetter;
                        }
                    }
                    ')' => {
                        state = State::DeepEnd;
                    }
                    '(' => {
                        parts_stack.push(parts);
                        parts = HashMap::new();
                        // It is guaranteed that name is an empty String, making no new allocation needed
                        state = State::DeepNone;
                    }
                    _ => return None,
                }
            }
            (State::CompositeDigit, '0'..='9') => {
                composite_count = composite_count * 10 + (c as usize - '0' as usize);
                state = State::CompositeDigit;
            }
            (State::CompositeDigit, _) => {
                let mut saved_parts = parts_stack
                    .pop()
                    .expect("State::CompositeDigit with empty saved_parts");
                for (name, count) in parts.iter() {
                    let count = count * composite_count;
                    if let Some(saved_count) = saved_parts.get_mut(name) {
                        *saved_count += count;
                    } else {
                        saved_parts.insert(name.clone(), count);
                    }
                }
                parts = saved_parts;
                match c {
                    'A'..='Z' => {
                        name.push(c);
                        if parts_stack.is_empty() {
                            state = State::ShallowLetter;
                        } else {
                            state = State::DeepLetter;
                        }
                    }
                    ')' => {
                        state = State::DeepEnd;
                    }
                    '(' => {
                        parts_stack.push(parts);
                        parts = HashMap::new();
                        // Refer to (State::DeepEnd, '(')
                        state = State::DeepNone;
                    }
                    _ => return None,
                }
            }
            _ => return None,
        }
    }
    match state {
        State::ShallowLetter => {
            create_or_add(&mut parts, name, 1);
        }
        State::ShallowDigit => {
            create_or_add(&mut parts, name, count);
        }
        State::DeepEnd => {
            let mut saved_parts = parts_stack
                .pop()
                .expect("State::DeepEnd with empty saved_parts");
            for (name, count) in parts.iter() {
                create_or_add(&mut saved_parts, name.clone(), *count);
            }
            parts = saved_parts;
        }
        State::CompositeDigit => {
            let mut saved_parts = parts_stack
                .pop()
                .expect("State::CompositeDigit with empty saved_parts");
            for (name, count) in parts.iter() {
                let count = count * composite_count;
                create_or_add(&mut saved_parts, name.clone(), count);
            }
            parts = saved_parts;
        }
        State::None => {}
        State::DeepNone | State::DeepLetter | State::DeepDigit => return None,
    }

    Some(Chemical {
        parts,
        display: input.into(),
    })
}

fn create_or_add(map: &mut HashMap<String, usize>, key: String, value: usize) {
    if let Some(previous_value) = map.get_mut(&key) {
        *previous_value += value;
    } else {
        map.insert(key, value);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_chemical_test_shallow() {
        let output = parse_chemical("CH3COONa").unwrap().parts;
        assert_eq!(2, output["C"]);
        assert_eq!(3, output["H"]);
        assert_eq!(2, output["O"]);
        assert_eq!(1, output["Na"]);
    }

    #[test]
    fn parse_chemical_test_deep() {
        let output = parse_chemical("(MgFe)2(MgFe)(OH)2Si8O22").unwrap().parts;
        assert_eq!(3, output["Mg"]);
        assert_eq!(3, output["Fe"]);
        assert_eq!(24, output["O"]);
        assert_eq!(2, output["H"]);
        assert_eq!(8, output["Si"]);
    }
}
