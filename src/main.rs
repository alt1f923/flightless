mod mathdaddy {
    let operators: [char; 4] = ['+', '-', '/', '*'];
    fn is_postfix(statement: &str) -> bool {operators.find(statement.chars().last().unwrap())};
    pub fn solve(statement: &str) {
        if is_postfix(statement) {

        }
    }
}
Arr
Array
arr
array 


fn main() {
    mathdaddy::solve()
}


// for each symbol s
//   if s is an operand
//     output s
//   else if it is a left parenthesis
//     push s
//   else if it is a right parenthesis
//     pop to output until corresponding left parenthesis popped
//   else # it is an operator
//     pop all higher or equal precedence operators (than s) to output
//     push s
// pop all remaining operators to output