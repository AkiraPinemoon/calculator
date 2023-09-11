fn main() {
    loop {
        println!("Type an expression!");
        let mut expression = String::new();
        std::io::stdin().read_line(&mut expression).unwrap();

        let tokenstream = lex(expression);
        println!("tokenstream: {:?}", tokenstream);

        let ast = parse(tokenstream);
        println!("abstract syntax tree: {:?}", ast);

        println!("reconstructed input: {}", string_from_ast(ast.clone()));

        let result = evaluate(ast);
        println!("= {}", result);
    }
}


#[derive(Debug, Clone)]
enum Token {
    Value(f64),
    Add,
    Substract,
    Multiply,
    Divide,
    Exponentiate,
    Brackets(Vec<Token>),
}

fn lex(s: String) -> Vec<Token> {
    let mut tokenstream = Vec::new();

    let mut current_token = String::new();
    let mut current_is_number = false;

    let mut bracket_count = 0;
    let mut bracket_data: Vec<Vec<Token>> = Vec::new();
    let mut current_out = &mut tokenstream;

    for ch in s.chars().into_iter() {
        match ch {
            ' ' => {},
            '1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9'|'0'|'.' => {
                current_is_number = true;
                current_token.push(ch)
            },
            ch => {
                if current_is_number {
                    current_out.push(Token::Value(current_token.parse().unwrap()));
                    current_is_number = false;
                    current_token = String::new();
                }

                match ch {
                    '+' => {current_out.push(Token::Add)},
                    '-' => {current_out.push(Token::Substract)},
                    '*' => {current_out.push(Token::Multiply)},
                    '/' => {current_out.push(Token::Divide)},
                    '^' => {current_out.push(Token::Exponentiate)},
                    '('|'['|'<'|'{' => {
                        bracket_count += 1;
                        bracket_data.push(Vec::new());
                        current_out = bracket_data.last_mut().unwrap()
                    },
                    ')'|']'|'>'|'}' => {
                        let new_token = Token::Brackets(bracket_data.pop().unwrap());

                        bracket_count -= 1;
                        if bracket_count == 0 {
                            current_out = &mut tokenstream;
                        } else {
                            current_out = bracket_data.last_mut().unwrap();
                        }

                        current_out.push(new_token);
                    },
                    _ => {},
                }
            },
        }
    }

    tokenstream
}

fn get_token_priority(token: Token) -> u8 {
    match token {
        Token::Add => 4,
        Token::Substract => 4,
        Token::Divide => 3,
        Token::Multiply => 3,
        Token::Exponentiate => 2,
        Token::Brackets(_) => 1,
        Token::Value(_) => 0,
    }
}

#[derive(Debug, Clone)]
enum Node {
    Value(f64),
    Addition(Box<Node>, Box<Node>),
    Substraction(Box<Node>, Box<Node>),
    Multiplication(Box<Node>, Box<Node>),
    Division(Box<Node>, Box<Node>),
    Exponentiation(Box<Node>, Box<Node>),
    Brackets(Box<Node>),
}

fn parse(tokenstream: Vec<Token>) -> Node {
    let highest_priority = tokenstream.iter().map(|token| {get_token_priority(token.clone())}).max().unwrap();

    for priority in (0..highest_priority + 1).rev() {
        for i in 0..tokenstream.len() {
            if get_token_priority(tokenstream[i].clone()) == priority {
                match tokenstream[i].clone() {
                    Token::Value(x) => return Node::Value(x),
                    Token::Add => {
                        let (left, right) = tokenstream.split_at(i);
                        return Node::Addition(Box::new(parse(left.into())), Box::new(parse(right[1..].into())));
                    },
                    Token::Substract => {
                        let (left, right) = tokenstream.split_at(i);
                        return Node::Substraction(Box::new(parse(left.into())), Box::new(parse(right[1..].into())));
                    },
                    Token::Multiply => {
                        let (left, right) = tokenstream.split_at(i);
                        return Node::Multiplication(Box::new(parse(left.into())), Box::new(parse(right[1..].into())));
                    },
                    Token::Divide => {
                        let (left, right) = tokenstream.split_at(i);
                        return Node::Division(Box::new(parse(left.into())), Box::new(parse(right[1..].into())));
                    },
                    Token::Exponentiate => {
                        let (left, right) = tokenstream.split_at(i);
                        return Node::Exponentiation(Box::new(parse(left.into())), Box::new(parse(right[1..].into())));
                    },
                    Token::Brackets(x) => return Node::Brackets(Box::new(parse(x))),
                    //Token::Brackets(x) => return parse(x), // this works too, but it's harder to reconstruct the input
                }

            }
        }
    }
    panic!()
}

fn deparse(ast: Node) -> Vec<Token> {
    match ast {
        Node::Value(x) => vec![Token::Value(x)],
        Node::Addition(x, y) => {
            let mut stream = deparse(*x);
            stream.push(Token::Add);
            stream.append(&mut deparse(*y));
            stream
        },
        Node::Substraction(x, y) => {
            let mut stream = deparse(*x);
            stream.push(Token::Substract);
            stream.append(&mut deparse(*y));
            stream
        },
        Node::Multiplication(x, y) => {
            let mut stream = deparse(*x);
            stream.push(Token::Multiply);
            stream.append(&mut deparse(*y));
            stream
        },
        Node::Division(x, y) => {
            let mut stream = deparse(*x);
            stream.push(Token::Divide);
            stream.append(&mut deparse(*y));
            stream
        },
        Node::Exponentiation(x, y) => {
            let mut stream = deparse(*x);
            stream.push(Token::Exponentiate);
            stream.append(&mut deparse(*y));
            stream
        },
        Node::Brackets(x) => vec![Token::Brackets(deparse(*x))]
    }
}

fn delex(tokenstream: Vec<Token>) -> String {
    let mut out = String::new();

    for token in tokenstream.into_iter() {
        match token {
            Token::Value(x) => out += &x.to_string(),
            Token::Add => out.push('+'),
            Token::Substract => out.push('-'),
            Token::Multiply => out.push('*'),
            Token::Divide => out.push('/'),
            Token::Exponentiate => out.push('^'),
            Token::Brackets(x) => {
                out.push('(');
                out += &delex(x);
                out.push(')');
            },
        }
    }

    out
}

fn string_from_ast(ast: Node) -> String {
    let tokenstream = deparse(ast);
    delex(tokenstream)
}

fn evaluate(node: Node) -> f64 {
    match node {
        Node::Value(x) => x,
        Node::Addition(x, y) => evaluate(*x) + evaluate(*y),
        Node::Substraction(x, y) => evaluate(*x) - evaluate(*y),
        Node::Multiplication(x, y) => evaluate(*x) * evaluate(*y),
        Node::Division(x, y) => evaluate(*x) / evaluate(*y),
        Node::Exponentiation(x, y) => evaluate(*x).powf(evaluate(*y)),
        Node::Brackets(x) => evaluate(*x),
    }
}
