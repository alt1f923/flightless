pub mod parser {
    use regex::Regex;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref OPERAND_CHAR_RE: Regex = Regex::new(r"\w").unwrap(); //Regular expression for 1 character 
        static ref OPERAND_STRING_RE: Regex =  Regex::new(r"\w+").unwrap(); //Regular expression for 1 or more characters
    }


    pub fn is_parenthesis(token: char) -> bool {
        match token {
            '(' | ')' => true,
            _ => false,
        }
    }

    pub fn is_operator(token: char) -> bool {
        match token {
            '+' | '-' | '/' | '*' | '^'  => true,
            _  => false,
        }
    }

    pub fn is_operand_variable(token: String) -> bool {OPERAND_STRING_RE.is_match(&token)}

    fn is_operand_character(token: char) -> bool {OPERAND_CHAR_RE.is_match(&token.to_string())}

    pub fn is_operand_literal(token: String) -> bool {token.parse::<f64>().is_ok()}

    pub fn is_operand(token: &String) -> bool {is_operand_literal(token.to_string()) || is_operand_variable(token.to_string()) }

    pub fn parse(statement: String) -> Vec<String> {
        let mut stack = Vec::<String>::new();
        let mut prev_operand: bool = false;

        for character in statement.chars() {
            if is_operand_character(character) { //If character is alphanumeric or an underscore (operands)
                if prev_operand { //If the previous is alphanumeric or an underscore (operand)
                    let mut operand: String = stack.pop().unwrap().to_string(); //Pop last item from stack as a String
                    operand.push(character); //Append the character to the end of the operand
                    stack.push(operand); //Push the edited operand to the stack

                }else { //If the previous is not alphanumeric or an underscore
                    let operand: String = character.to_string();
                    stack.push(operand);
                    prev_operand = true; //Since this character was the first character of a possible many, this will help to catch the rest of the operand in future sequential iterations

                }
            }else if is_parenthesis(character) || is_operator(character) { //If character is a parenthesis or operator
                stack.push(character.to_string());
                prev_operand = false;

            }else {
                prev_operand = false;

            }
        }
        stack //Return the stack
    }
}


mod notation {
    use crate::mathdaddy::parser;

    fn priority(operator: &str) -> u8 {
        match operator {
            "+" => 1,
            "-" => 0,
            "/" => 3,
            "*" => 2,
            "^" => 4,
             _  => 0, //This might be a bad inclusion, revisit later
        }
    }

    fn do_operation(operator: &str, x: f64, y: f64) -> String {
        match operator {
            "+" => x + y,
            "-" => x - y,
            "/" => x / y,
            "*" => x * y,
            "^" => x.powf(y),
             _  => 0.0, //This might be a bad inclusion, revisit later
        }.to_string()
    }

    fn is_postfix(statement: &String) -> bool { //Postfix expressions have operators after numbers, hence the last character of such an expression will be an operator
        parser::is_operator(statement.chars().last().unwrap()) //Returning a boolean of whether the last character is of the String is an operator
    }

    fn is_prefix(statement: &String) -> bool { //Prefix expressions have operators before numbers, hence the first character of such an expression will be an operator
        parser::is_operator(statement.chars().nth(0).unwrap()) //Returning a boolean of whether the first character is of the String is an operator
    }

    fn postfix_to_postfix(statement: &String) -> Vec<String> { //This may sound redundant, but this will be returning the same statement AFTER the parser has gone through it, this will format the statement
        parser::parse(statement.to_string()) //Returning the iterator
    }
    
    fn infix_to_postfix(statement: &String) -> Vec<String> {
        let mut opstack = Vec::<String>::new();
        let mut stack = Vec::<String>::new();

        for token in parser::parse(statement.to_string()).iter() {
            if parser::is_operand(token) {stack.push(token.to_string());}
            else {
                match token as &str {
                    "(" => opstack.push(token.to_string()),
                    ")" => {let mut operator = stack.pop().unwrap(); 
                            while operator != "(".to_string() { //Popping all operands and operators out of the stack until the accompying parenthesis is found
                                stack.push(operator);
                                operator = opstack.pop().unwrap();
                            }},//Assuming if it's not a parenthesis or operand, it will be an operator. True in valid infix notation
                     _  => {while opstack.len() > 0 && priority(token) <= priority(&opstack[opstack.len()-1]) { //Popping all operators until the stack is empty or an operator of lower importance is found
                                stack.push(opstack.pop().unwrap());
                            }
                            opstack.push(token.to_string()); //Push operator onto stack
        }}}}
        while opstack.len() > 0 {//Pushing all remaining operators off the stack and onto the end of the equation
            stack.push(opstack.pop().unwrap());
    }
    stack //Returning stack
    }

    fn prefix_to_postfix(statement: &String) -> Vec<String> {
        let mut stack = Vec::<String>::new(); // Change vector defintiions to vec![] soon
        for token in parser::parse(statement.to_string()).iter().rev() { //Reading the statement from right to left (reverse order)
            if parser::is_operand(token) {
                stack.push(token.to_string());
            }else{ //Token is not an operand, so assuming this is a valid prefix equation it will be an operator
                let x = stack.pop().unwrap();
                let y = stack.pop().unwrap();
                stack.push(format!("{} {} {}", x, y, token));
                // stack.append(&mut vec![x, y, token.to_string()]);
            }
        }
        str::split_whitespace(&stack.pop().unwrap()).map(|x| x.to_string()).collect::<Vec<String>>() //Returns the stacks only item, split by whitespaces, with each item being converted to a String from &str
    }

    fn solve_postfix(stack: Vec<String>) -> String {
        let mut opstack = Vec::<String>::new();

        for token in &stack {
            if parser::is_operand_literal(token.to_string()) { //Operand as a number 
                opstack.push(token.to_string()); //Turn the number that is expressed as a string into a 64 bit float integer
            }else if parser::is_operand_variable(token.to_string()) {
                opstack.push(token.to_string());
            }else { //Assuming statement is in correct postfix notation, if it is not an operand it will be an operator
                let y = opstack.pop().unwrap();
                let x = opstack.pop().unwrap();
                
                opstack.push(do_operation(&token, x.parse::<f64>().unwrap(), y.parse::<f64>().unwrap()));
            }

        }
        opstack.pop().unwrap()
    }

    fn stack_to_string(stack: &Vec<String>) -> String {
        let mut output = String::new();
        for token in stack {
            output.push_str(&token);
            output.push(' ');
        }
        output //Returning the String output
    }

    pub fn solve(statement: &String) -> (String, String, String) {

        let postfix_stack = if is_postfix(statement) {postfix_to_postfix(statement)} 
                     else if is_prefix(statement) {prefix_to_postfix(statement)}
                     else {infix_to_postfix(statement)};
        let postfix_string = stack_to_string(&postfix_stack);
        (solve_postfix(postfix_stack), statement.to_string(), postfix_string) //Returning a tuple of the solution, original statement and what it was converted to (postfix)
    }
}


/// Do not include in under notation - This function will be expanded upon in the future to check for future question types
pub fn solve(statement: &String) -> (String, String, String) {
    notation::solve(statement)
}