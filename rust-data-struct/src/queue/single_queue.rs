use std::vec::Vec;
use std::option::Option::Some;
use std::option::Option::None;
use std::option::Option;

#[derive(Debug)]
pub struct SingleQueue<T> {
    pub data: Vec<T>,
    pub font: i32,
    pub rear: i32,
    pub size: i32,
}

impl<T> SingleQueue<T> {
    pub fn new() -> SingleQueue<T> {
        SingleQueue {
            data: Vec::new(),
            font: 0,
            rear: 0,
            size: 0,
        }
    }

    pub fn add(&mut self, param: T) {
        self.data.push(param);
        self.rear += 1;
        self.size += 1;
    }

    pub fn size(&self) -> i32 {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        if self.size == 0 {
            true
        } else {
            false
        }
    }

    pub fn pool(&mut self) -> Option<T> {
        if self.size > 0 {
            let a = self.data.remove(0);
            self.size -= 1;
            self.font += 1;
            Some(a)
        } else {
            None
        }
    }
}