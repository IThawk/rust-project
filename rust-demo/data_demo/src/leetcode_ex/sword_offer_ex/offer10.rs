use std::collections::HashMap;


struct Solution {
}

/**
*
* 写一个函数，输入 n ，求斐波那契（Fibonacci）数列的第 n 项（即 F(N)）。斐波那契数列的定义如下：

F(0) = 0,
F(1)= 1
F(N) = F(N - 1) + F(N - 2), 其中 N > 1.
斐波那契数列由 0 和 1 开始，之后的斐波那契数就是由之前的两数相加而得出。

答案需要取模 1e9+7（1000000007），如计算初始结果为：1000000008，请返回 1。

来源：力扣（LeetCode）
链接：https://leetcode-cn.com/problems/fei-bo-na-qi-shu-lie-lcof
著作权归领扣网络所有。商业转载请联系官方授权，非商业转载请注明出处。
*/
impl Solution {
    pub fn fib(n: i32) -> i32 {
        let mut dp = Vec::new();
        for i in 0..(n+1) {
            if i == 0 {
                dp.push(0);
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