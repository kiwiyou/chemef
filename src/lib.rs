mod chemical;
mod reaction;

use chemical::*;
use reaction::ReactionError;
use seed::{prelude::*, *};

struct Model {
    pub input: String,
    pub result: Option<Vec<FormattedChemical>>,
    pub error: Option<String>,
    pub history: Vec<(Vec<FormattedChemical>, Vec<FormattedChemical>)>,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            input: String::new(),
            result: None,
            error: None,
            history: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Msg {
    Balance,
    InputKeyDown(String),
    SetInput(String),
    Reset,
    Idle,
}

fn update(msg: Msg, model: &mut Model, order: &mut impl Orders<Msg>) {
    match msg {
        Msg::InputKeyDown(key_string) => {
            if key_string == "Enter" {
                order.skip();
                order.send_msg(Msg::Balance);
            }
        }
        Msg::Balance => {
            model.error = None;
            match parse_equation(&model.input) {
                Ok((reagents, products)) => {
                    match reaction::calculate_coefficients(&reagents, &products) {
                        Ok(coefficients) => {
                            let mut result = Vec::new();
                            let mut is_first = true;
                            for (reagent, coef) in reagents.iter().zip(coefficients.iter()) {
                                if !is_first {
                                    result.push(FormattedChemical::Text(" + ".into()));
                                } else {
                                    is_first = false;
                                }
                                if *coef > 1 {
                                    result.push(FormattedChemical::Bold(coef.to_string()));
                                }
                                result.append(&mut format_chemicals(&reagent.display));
                            }
                            result.push(FormattedChemical::Text(" = ".into()));
                            let skipped = coefficients.iter().skip(reagents.len());
                            is_first = true;
                            for (product, coef) in products.iter().zip(skipped) {
                                if !is_first {
                                    result.push(FormattedChemical::Text(" + ".into()));
                                } else {
                                    is_first = false;
                                }
                                if *coef > 1 {
                                    result.push(FormattedChemical::Bold(coef.to_string()));
                                }
                                result.append(&mut format_chemicals(&product.display));
                            }
                            model.result = Some(result.clone());
                            model.history.push((format_chemicals(&model.input), result));
                            model.input.clear();
                        }
                        Err(ReactionError::InfiniteSolution) => {
                            model.error = Some("계수가 하나로 정해지지 않습니다.".into())
                        }
                        Err(ReactionError::UnbalancedElements) => {
                            model.error = Some(
                                "반응물의 원소 종류와 생성물의 원소 종류가 일치하지 않습니다."
                                    .into(),
                            )
                        }
                    }
                }
                Err(error) => model.error = Some(error),
            }
            order.after_next_render(|_| {
                activate_all_animations();
                Msg::Idle
            });
        }
        Msg::SetInput(input) => model.input = input,
        Msg::Reset => {
            model.result = None;
            model.error = None;
        }
        Msg::Idle => {
            order.skip();
        }
    }
}

fn parse_equation(input: impl AsRef<str>) -> Result<(Vec<Chemical>, Vec<Chemical>), String> {
    let input = input.as_ref();
    let mut split = input.splitn(2, '=');
    let mut reagents = Vec::new();
    let left = split.next().unwrap();
    for reagent_str in left.split('+') {
        let reagent = parse_chemical(reagent_str.trim())
            .ok_or(format!("{}은(는) 올바른 화학식이 아닙니다.", reagent_str));
        reagents.push(reagent?);
    }
    let right = split
        .next()
        .ok_or("반응물1 + 반응물2 + ... = 생성물1 + 생성물2 + ... 형식으로 입력해주세요.");
    let mut products = Vec::new();
    for product_str in right?.split('+') {
        let product = parse_chemical(product_str.trim())
            .ok_or(format!("{}은(는) 올바른 화학식이 아닙니다.", product_str));
        products.push(product?);
    }
    Ok((reagents, products))
}

#[derive(Debug, Clone, PartialEq)]
enum FormattedChemical {
    Bold(String),
    Text(String),
    Sub(String),
}

impl FormattedChemical {
    fn node(&self) -> Node<Msg> {
        match self {
            FormattedChemical::Bold(s) => b! { s },
            FormattedChemical::Text(s) => Node::new_text(s.clone()),
            FormattedChemical::Sub(s) => sub! { s },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_chem() {
        assert_eq!(
            vec![
                FormattedChemical::Text("H".into()),
                FormattedChemical::Sub("2".into()),
            ],
            format_chemicals("H2")
        );
    }
}

fn format_chemicals(chemical: &str) -> Vec<FormattedChemical> {
    let mut components: Vec<FormattedChemical> = Vec::new();
    let mut stage = chemical;
    loop {
        if let Some(index) = stage.find(|c: char| c.is_numeric()) {
            components.push(FormattedChemical::Text(stage[..index].into()));
            stage = &stage[index..];
        } else {
            break;
        }
        if let Some(index) = stage.find(|c: char| !c.is_numeric()) {
            components.push(FormattedChemical::Sub(stage[..index].into()));
            stage = &stage[index..];
        } else {
            components.push(FormattedChemical::Sub(stage.into()));
            stage = &stage[stage.len()..];
        }
    }
    if !stage.is_empty() {
        components.push(FormattedChemical::Text(stage.into()));
    }
    components
}

fn how_to_view() -> Node<Msg> {
    header! {
        attrs! {
            At::Id => "how-to",
        },
        h1! { "How to use?" },
        p! {
            "입력칸에 A + B = C + D와 같은 형태로 계수를 맞출 반응식을 작성합니다.",
            br! {},
            "(g), (aq)와 같은 물질의 상태는 작성하지 말아 주세요."
        },
    }
}

fn input_animation(model: &Model) -> Node<Msg> {
    let class = if model.error.is_some() {
        class!["error"]
    } else if model.result.is_some() {
        class!["ok"]
    } else {
        class![]
    };

    let animation = if model.error.is_some() || model.result.is_some() {
        vec![
            animate! {
                attrs! {
                    At::Custom("attributeName".into()) => "r",
                    At::Custom("values".into()) => "0;100%",
                    At::Custom("dur".into()) => "0.5s",
                    At::Custom("begin".into()) => "indefinite",
                },
            },
            animate! {
                attrs! {
                    At::Custom("attributeName".into()) => "opacity",
                    At::Custom("values".into()) => "1;0",
                    At::Custom("dur".into()) => "0.5s",
                    At::Custom("begin".into()) => "indefinite",
                },
            },
        ]
    } else {
        vec![]
    };

    svg! {
        class,
        attrs! {
            At::Custom("preserveAspectRatio".into()) => "none",
        },
        rect! {
            attrs! {
                At::Width => "100%",
                At::Height => "100%",
                At::Fill => "white",
            }
        },
        circle! {
            attrs! {
                At::Custom("cx".into()) => "50%",
                At::Custom("cy".into()) => "50%",
                At::Custom("r".into()) => "0",
            },
            animation,
        }
    }
}

fn input_view(model: &Model) -> Node<Msg> {
    let expression_view = if let Some(ref result) = model.result {
        div![
            class!["result"],
            result.iter().map(FormattedChemical::node),
            simple_ev(Ev::Click, Msg::Reset),
        ]
    } else {
        input![
            attrs! {
                At::Name => "expression",
                At::Type => "text",
                At::Placeholder => "H2O = H2 + O2",
                At::Value => model.input,
                At::Custom("autofocus".into()) => "",
            },
            keyboard_ev("keydown", |ev| Msg::InputKeyDown(ev.key())),
            input_ev(Ev::Input, Msg::SetInput)
        ]
    };

    div![
        class!["expression"],
        input_animation(model),
        expression_view,
    ]
}

fn history_view(model: &Model) -> Node<Msg> {
    let mut list = Vec::new();
    for (index, (input, output)) in model.history.iter().enumerate() {
        list.push(li! {
            header! {
                format!("In[{}] : ", index)
            },
            section! {
                input.iter().map(FormattedChemical::node)
            }
        });
        list.push(li! {
            header! {
                format!("Out[{}] : ", index)
            },
            section! {
                output.iter().map(FormattedChemical::node)
            }
        });
    }
    ul! {
        class! [ "result" ],
        list
    }
}

fn view(model: &Model) -> impl IntoNodes<Msg> {
    let error_view = if let Some(ref error_message) = model.error {
        label![class!["error"], format!("Error : {}", error_message)]
    } else {
        empty![]
    };

    vec![
        how_to_view(),
        main! {
            id! { "calculator" },
            h1! { "반응식 균형 계산기" },
            input_view(model),
        },
        error_view,
        history_view(model),
    ]
}

#[wasm_bindgen]
extern "C" {
    fn activate_all_animations();
}

#[wasm_bindgen(start)]
pub fn render() {
    App::builder(update, view).build_and_start();
}
