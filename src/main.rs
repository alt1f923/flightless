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
             _  => 4,//This might be a bad inclusion, revisit later
        }
    }

    fn do_operation(operator: &str, x: f64, y: f64) -> f64 {
        match operator {
            "+" => x + y,
            "-" => x - y,
            "/" => x / y,
            "*" => x * y,
             _  => 0.0,//This might be a bad inclusion, revisit later
        }
    }

    fn is_postfix(statement: &String) -> bool {//Postfix expressions have operators after numbers, hence the last character of such an expression will be an operator
        is_operator(statement.chars().last().unwrap())
    }

    fn is_prefix(statement: &String) -> bool {//Prefix expressions have operators before numbers, hence the first character of such an expression will be an operator
        is_operator(statement.chars().nth(0).unwrap())
    }

    fn infix_to_postfix(statement: &String) -> String {
        let mut opstack = Vec::new();
        let mut output = std::string::String::new();

        for token in str::split_whitespace(&statement) {
            if is_operand(token) {
                output = output + token;
                output.push(' ')
            }else {//Not an operand, however could be an operand inside parenthesises or an operator
                    if token.chars().nth(0).unwrap() == '(' {//Checking if first character of this string is a left parenthesis
                        opstack.push("(");
                        opstack.push(&token[1..token.len()]);
                    } else if token.chars().last().unwrap() == ')' {//Checking if last character of this string is a right parenthesis
                        opstack.push(&token[0..token.len()-1]);
                        let mut operator = opstack.pop().unwrap();
                        while operator != "(" {//Popping all operands and operators out of the stack until the accompying parenthesis is found
                            output = output + operator;
                            output.push(' ');
                            operator = opstack.pop().unwrap();
                        }
                    } else {//Assuming if it's not a parenthesis or operand, it will be an operator. True in valid infix notation
                        
                        while opstack.len() > 0 && priority(token) <= priority(opstack[opstack.len()-1]) {//Popping all operators until the stack is empty or an operator of lower importance is found
                            output = output + opstack.pop().unwrap();
                            output.push(' ');
                        
                        }
                    opstack.push(token);//Push operator onto stack
                    }
            } 
        }
        while opstack.len() > 0 {//Pushing all remaining operators off the stack and onto the end of the equation
            output = output + opstack.pop().unwrap();
            output.push(' ');
        }
        output
    }

    fn solve_postfix(statement: &String) -> f64 {
        let mut opstack = Vec::new();

        for token in str::split_whitespace(&statement) {
            if is_operand(token) {
                opstack.push(token.parse::<f64>().unwrap());//Turn the number that is expressed as a string into a 64 bit float integer
            }else {//Assuming statement is in correct postfix notation, if it is not an operand it will be an operator
                let y = opstack.pop().unwrap();
                let x = opstack.pop().unwrap();
                opstack.push(do_operation(token, x, y));
            }

        }
        opstack.pop().unwrap()
    }

    pub fn solve(statement: &String) -> (f64, String, String) {
        let postfix_eq = if is_postfix(statement) {statement.to_string()}
                         else if is_prefix(statement) {std::string::String::from("3 2 +")}
                         else {infix_to_postfix(statement)};
        (solve_postfix(&postfix_eq), statement.to_string(), postfix_eq)
    }

}



fn main() {
    let x = std::string::String::from("3 2 +");
    println!("{:?}", mathdaddy::solve(&x));
    
    let y = std::string::String::from("+ 3 2");
    println!("{:?}", mathdaddy::solve(&y));
    
    let z = std::string::String::from("3 + 2");
    println!("{:?}", mathdaddy::solve(&z));
}