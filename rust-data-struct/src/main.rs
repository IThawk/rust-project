mod Algorithms;
mod linked_list;
mod queue;
mod recursion;
mod two_arrays;
use queue::SingleQueue;
use recursion::factorial;
use std::string::String;

fn main() {
    let mut a: SingleQueue<String> = SingleQueue::new(2);
    a.add(String::from("ithawk"));
    println!("Hello, world!{:?}", a.len());

    factorial::factorial_main();
}
