mod mathdaddy {
    fn is_operator(character: char) -> bool {
        match character {
            '+' => true,
            '-' => true,
            '/' => true,
            '*' => true,
             _  => false,
        }
    }
    fn is_postfix(statement: &str) -> bool {
        is_operator(statement.chars().last().unwrap())
    }
    fn is_prefix(statement: &str) -> bool {
        is_operator(statement.chars().nth(0).unwrap())
    }
    fn postfix
    pub fn solve(statement: &str) {
        if is_postfix(statement) {
            println!("postfix");
        } else if is_prefix(statement) {
            println!("prefix");
        } else {
            println!("infix");
        }
    }
}


fn main() {
    mathdaddy::solve("3 2 +");
    mathdaddy::solve("+ 3 2");
    mathdaddy::solve("3 + 2");
}