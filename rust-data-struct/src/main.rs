mod queue;
mod two_arrays;
use queue::SingleQueue;
use std::string::String;

fn main() {
    let mut a : SingleQueue<String> = SingleQueue::new(2);
    a.add(String::from("ithawk"));
    println!("Hello, world!{:?}",a.len());
}
