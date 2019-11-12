use super::single_queue::{SingleQueue};
use std::string::String;

#[test]
fn test_new_single_queue() {

   let a:SingleQueue<String> = SingleQueue::new(3);
}