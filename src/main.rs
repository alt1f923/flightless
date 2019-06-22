mod mathdaddy {
    fn is_operator(token: char) -> bool {
        match token {
            '+' | '-' | '/' | '*' => true,
             _  => false,
        }
    }
    fn is_operand(token: &str) -> bool {
        token.parse::<f64>().is_ok()
    }
    fn priority(operator: &str) -> u8 {
        match operator {
            "+" => 1,
            "-" => 0,
            "/" => 3,
            "*" => 2,
             _  => 4,
        }
    }
    fn is_postfix(statement: &String) -> bool {
        is_operator(statement.chars().last().unwrap())
    }
    fn is_prefix(statement: &String) -> bool {
        is_operator(statement.chars().nth(0).unwrap())
    }

    fn infix_to_postfix(statement: &String) -> String {
        let mut opstack = Vec::new();
        let mut output = std::string::String::new();

        for token in str::split_whitespace(&statement) {
            if is_operand(token) {
                output = output + token;
                output.push(' ')
            }else {
                    if token.chars().nth(0).unwrap() == '(' {
                        opstack.push("(");
                        opstack.push(&token[1..token.len()]);
                    } else if token.chars().last().unwrap() == ')' {
                        opstack.push(&token[0..token.len()-1]);
                        let mut operator = opstack.pop().unwrap();
                        while operator != "(" {
                            output = output + operator;
                            output.push(' ');
                            operator = opstack.pop().unwrap();
                        }
                    } else {
                        while opstack.len() > 0 && priority(token) <= priority(opstack[opstack.len()-1]) {
                            output = output + opstack.pop().unwrap();
                            output.push(' ');
                        opstack.push(token);
                        }
                    }
            } 
        }
        while opstack.len() > 0 {
            println!("{:?}", opstack);
            output = output + opstack.pop().unwrap();
            output.push(' ');
        }
        
        output
    }
    pub fn solve(statement: &String) {
        if is_postfix(statement) {
            println!("postfix");
        } else if is_prefix(statement) {
            println!("prefix");
        } else {
            println!("infix statement: {} => postfix conversion: {}", statement, infix_to_postfix(statement));
        }
    }
}



fn main() {
    let x = std::string::String::from("3 2 +");
    mathdaddy::solve(&x);
    
    let y = std::string::String::from("+ 3 2");
    mathdaddy::solve(&y);
    
    let z = std::string::String::from("3 + 2");
    mathdaddy::solve(&z);
}