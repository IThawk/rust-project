use std::sync::{Arc,Mutex};

pub fn arc_main(){
    let a = Arc::new(2);
    println!("{:?}",a.clone());
    let uer1 = User{
       name : String::from("lili"),
        id :12
    };
    let b = Arc::new(uer1);
    println!("{:?}",b.clone());
    let c = Arc::new(User{
        name:String::from("xiaohong"),
        id:13
    });
    let d = Mutex::new(c);
    let mut  e = Arc::new(d);
    let mut f = e.lock().unwrap().clone();

    let user2 = User{
        name:String::from("xiaoming"),
        id:14
    };
//    let mut user = f.clone();
    f = Arc::new(user2);
    let g = Arc::new( Mutex::new(f));

    println!("f:{:?}",g.clone().lock().unwrap())
}
#[derive(Debug)]
struct User{
    name : String,
    id :i32
}