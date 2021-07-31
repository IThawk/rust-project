/**
*
* 用两个栈实现一个队列。队列的声明如下，请实现它的两个函数 appendTail 和 deleteHead ，分别完成在队列尾部插入整数和在队列头部删除整数的功能。(若队列中没有元素，deleteHead 操作返回 -1 )

* 来源：力扣（LeetCode）
* 链接：https://leetcode-cn.com/problems/yong-liang-ge-zhan-shi-xian-dui-lie-lcof
* 著作权归领扣网络所有。商业转载请联系官方授权，非商业转载请注明出处。
* VecDeque :队列
*/
use std::collections::VecDeque;
struct CQueue {
    vec : VecDeque<i32>,
}

/**
 * `&self` means the method takes an immutable reference.
 * If you need a mutable reference, change it to `&mut self` instead.
 */
impl CQueue {

    fn new() -> Self {
        CQueue { vec : VecDeque::new() }
    }

    fn append_tail(&mut self, value: i32) {
        self.vec.push_back(value);
    }

    fn delete_head(&mut self) -> i32 {
        if self.vec.is_empty() {
            return -1;
        }
        if let Some(n) = self.vec.pop_front() {
            return n;
        }
        -1
    }
}

/**
 * Your CQueue object will be instantiated and called as such:
 * let obj = CQueue::new();
 * obj.append_tail(value);
 * let ret_2: i32 = obj.delete_head();
 */
#[test]
pub fn main_test() {
    let mut obj = CQueue::new();
    obj.append_tail(1);
    let ret_2: i32 = obj.delete_head();
    println!("delete_head:{}", ret_2);
}