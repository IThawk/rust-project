use std::ptr::NonNull;

fn main() {
    let x: &'static NonNull<u32> = &(NonNull::dangling());
    //~^ ERROR borrowed value does not live long enough

    let mut i: i32 = 10;
    let non_null = NonNull::new(&mut i).unwrap();
    let x: &'static NonNull<u32> = &(non_null.cast());
    //~^ ERROR borrowed value does not live long enough
}
