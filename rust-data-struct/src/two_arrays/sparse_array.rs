use super::single_two_arrays::TwoArrays;
use std::clone::Clone;
use std::default::Default;
use std::fmt::Debug;
use std::option::Option;
use std::option::Option::None;
use std::option::Option::Some;
use std::vec::Vec;

static DEFAULT_SIZE: usize = 8;

#[derive(Debug)]
pub struct SparseArray {
    pub data: Vec<Vec<i32>>,
}

impl SparseArray {
    pub fn new(x: usize) -> SparseArray {
        SparseArray {
            data: vec![vec![0; x]; 3],
        }
    }
    pub fn from(&mut self, array: TwoArrays<i32>) {}
}
