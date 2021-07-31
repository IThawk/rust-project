use crate::linked_list::single_linked_list::SingleLinedList;
use std::string::String;

#[test]
fn test_new_single_linked_list() {
    let a: SingleLinedList<String> = SingleLinedList::new(3);
}
