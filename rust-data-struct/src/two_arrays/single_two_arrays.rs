use super::sparse_array::{SparseArray};
use std::vec::Vec;
use std::option::Option::Some;
use std::option::Option::None;
use std::option::Option;
use std::clone::Clone;
use std::default::Default;
use std::fmt::Debug;

static DEFAULT_SIZE: usize = 8;

#[derive(Debug)]
pub struct TwoArrays<T> {
    pub data: Vec<Vec<T>>,
    pub x: usize,
    pub y: usize
}

impl<T> TwoArrays<T>
    where T: Clone + Default + Debug
{
    pub fn new(x: usize,y:usize) -> TwoArrays<T> {
        TwoArrays {
            data:vec![vec![T::default(); x];y] ,
            x,
            y,
        }
    }

}

impl<T> Default for TwoArrays<T> where T: Clone + Default + Debug {
    fn default() -> TwoArrays<T> {
        TwoArrays {
            data:vec![vec![T::default(); DEFAULT_SIZE];DEFAULT_SIZE] ,
            x:DEFAULT_SIZE,
            y:DEFAULT_SIZE,
        }
    }
}

impl TwoArrays<i32> {
    fn default() -> TwoArrays<i32> {
        TwoArrays {
            data:vec![vec![0; DEFAULT_SIZE];DEFAULT_SIZE] ,
            x:DEFAULT_SIZE,
            y:DEFAULT_SIZE,
        }
    }
}