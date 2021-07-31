use std::collections::{HashMap, HashSet};
use hyper::http::header::IF_MATCH;


struct Solution {}

/**
*
把一个数组最开始的若干个元素搬到数组的末尾，我们称之为数组的旋转。输入一个递增排序的数组的一个旋转，输出旋转数组的最小元素。例如，数组 [3,4,5,1,2] 为 [1,2,3,4,5] 的一个旋转，该数组的最小值为1。

来源：力扣（LeetCode）
链接：https://leetcode-cn.com/problems/xuan-zhuan-shu-zu-de-zui-xiao-shu-zi-lcof
著作权归领扣网络所有。商业转载请联系官方授权，非商业转载请注明出处。
*/
impl Solution {
    pub fn exist(board: Vec<Vec<char>>, word: String) -> bool {
        let mut runEx = HashSet::new();
        let mut v = Vec::new();
        //找到包含第一个

        for i in (0..board.len()) {
            for j in board.get(i) {
                if word.chars()[0] == j {
                    v.push(i + "-" + j);
                    runEx.insert(i + "-" + j);
                    break;
                }
            }
        }
        if v.len() == 0 {
            return false;
        }
        for i in (1..word.len()) {
            let index_op = v.get(i-1);
            let a:String =match index_op{
                Some(i) => i.to_string(),
                None=>String::new()
            };
            if a=="" {
                return false;
            }
            let x_i = a.split("-");
            let a = x_i[0] as usize;
            let b = x_i[1] as usize;
            //向上走
            if a>0 {
                let aT = a-1;
                if  (!runEx.contains(aT+"-"+b)){

                }
            }

        }
        return false;
    }
}


#[test]
fn test() {
    let mut a: Vec<i32> = Vec::new();
    a.push(2);
    a.push(2);
    a.push(2);
    a.push(0);
    a.push(1);
    println!("{}", Solution::exist(a));
}