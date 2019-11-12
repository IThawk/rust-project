use std::vec::Vec;
use std::option::Option::Some;
use std::option::Option::None;
use std::option::Option;
use std::clone::Clone;
use std::default::Default;
use std::fmt::Debug;

static DEFAULT_SIZE: usize = 8;

#[derive(Debug)]
pub struct Stack<T> {
    pub data: Vec<T>,
    pub pointer: i32,
    pub size: usize,
}

impl<T> Stack<T>
    where T: Clone + Default + Debug
{
    pub fn new(size: usize) -> Stack<T> {
        Stack {
            data: vec![T::default(); size],
            pointer: -1,
            size,
        }
    }

    pub fn add(&mut self, param: T) -> bool {
        if self.is_full() {
            false
        } else {

            self.pointer += 1;
            let end = self.pointer as usize;
            self.data[end] = param;
            true
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        if self.font == -1 {
            true
        } else {
            false
        }
    }


    pub fn is_full(&self) -> bool {
        let size = self.pointer + 1;
        let size = size as usize;
        if size == self.size {
            true
        } else {
            false
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            let pointer = self.pointer as usize;
            let a = self.data[pointer].clone();
            self.data[pointer] = T::default();
            self.pointer -= 1;
            Some(a)
        }
    }
}

impl<T> Default for Stack<T> where T: Clone + Default + Debug {
    fn default() -> Stack<T> {
        Stack {
            data: vec![T::default(); DEFAULT_SIZE],
            pointer: -1,
            size: 8,
        }
    }
}