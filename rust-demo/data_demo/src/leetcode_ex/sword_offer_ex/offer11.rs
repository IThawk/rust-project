use std::collections::HashMap;


struct Solution {
}

/**
*
把一个数组最开始的若干个元素搬到数组的末尾，我们称之为数组的旋转。输入一个递增排序的数组的一个旋转，输出旋转数组的最小元素。例如，数组 [3,4,5,1,2] 为 [1,2,3,4,5] 的一个旋转，该数组的最小值为1。

来源：力扣（LeetCode）
链接：https://leetcode-cn.com/problems/xuan-zhuan-shu-zu-de-zui-xiao-shu-zi-lcof
著作权归领扣网络所有。商业转载请联系官方授权，非商业转载请注明出处。
*/
impl Solution {
    pub fn min_array(numbers: Vec<i32>) -> i32 {
        if numbers.len()==0 {
            return 0;
        }
        let mut min = numbers[0];
        if numbers.len()==1 {
            return numbers[0];
        }
        for i in (1..numbers.len()){
            if min>=numbers[i] {
                min = numbers[i];
            }
            if (numbers[i-1]>numbers[i]) {  break;}

        }
        return min;
    }
}


#[test]
fn test() {
    let mut a: Vec<i32> =   Vec::new();
    a.push(2);
    a.push(2);
    a.push(2);
    a.push(0);
    a.push(1);
    println!("{}", Solution::min_array(a));
}