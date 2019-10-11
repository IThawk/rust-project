mod queue;

use queue::SingleQueue;
use std::string::String;

fn main() {
    let mut a : SingleQueue<String> = SingleQueue::new();
    a.add(String::from("ithawk"));
    println!("Hello, world!{:?}",a.size());
}
