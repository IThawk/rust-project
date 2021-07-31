pub fn factorial_main() {
    let i = 5;
    let a = factorial_i32(i);
    println!("{}的阶乘是：{}", i, a);
}

///这个是递归求阶乘
pub fn factorial_i32(a: i32) -> i32 {
    if a == 1 {
        return 1;
    } else {
        return factorial_i32(a - 1) * a;
    }
}
