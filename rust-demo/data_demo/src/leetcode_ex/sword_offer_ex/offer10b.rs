use std::collections::HashMap;


struct Solution {
}

/**
*
一只青蛙一次可以跳上1级台阶，也可以跳上2级台阶。求该青蛙跳上一个 n级的台阶总共有多少种跳法。

答案需要取模 1e9+7（1000000007），如计算初始结果为：1000000008，请返回 1。

来源：力扣（LeetCode）
链接：https://leetcode-cn.com/problems/qing-wa-tiao-tai-jie-wen-ti-lcof
著作权归领扣网络所有。商业转载请联系官方授权，非商业转载请注明出处。
*/
impl Solution {
    pub fn fib(n: i32) -> i32 {
        let mut dp = Vec::new();
        for i in 0..(n+1) {
            if i == 0 {
                dp.push(1);
            } else if i==1 {
                dp.push(1);
            } else {
                dp.push((dp[(i-1) as usize]+dp[(i-2) as usize])%1000000007);
            }
        }
        dp[n as usize]
    }
}


pub fn fib(n: i32) -> i32 {
    if (n <= 1) {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}

#[test]
fn test() {
    println!("{}", Solution::fib(11));
    println!("{}", fib(1))
}