mod queue;
mod two_arrays;
mod recursion;
use queue::SingleQueue;
use std::string::String;
use recursion::factorial;

fn main() {
    let mut a : SingleQueue<String> = SingleQueue::new(2);
    a.add(String::from("ithawk"));
    println!("Hello, world!{:?}",a.len());

    factorial::factorial_main();
}
