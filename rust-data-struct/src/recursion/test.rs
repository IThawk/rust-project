use crate::recursion::factorial;

///阶乘的测试
#[test]
fn test_factorial() {
    let a = 5;
    let b = factorial::factorial_i32(a);
    assert_eq!(120, b)
}
