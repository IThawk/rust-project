use super::stack::{Stack};
use std::string::String;

#[test]
fn test_new_stack() {

   let a:Stack<String> = Stack::new(3);
   a.add(String::new());

}