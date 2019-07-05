use arrayvec::ArrayString;

#[derive(Debug,Ord, PartialOrd, Eq, PartialEq)]
struct User{
    name:String,
    id :ArrayString<[u8; 63]>

}
pub fn array_string_main(){
    test();
}

fn test(){
    let mut users :Vec<User> =  Vec::new();
    users.push(init_user("g1","xiaohong"));
    users.push(init_user("h2","xiaohong"));
    users.push(init_user("ff","xiaohong"));
    users.push(init_user("kk","xiaohong"));
    users.push(init_user("jj","xiaohong"));
    users.push(init_user("q3","xiaohong"));
    users.push(init_user("aa","xiaohong"));
    users.push(init_user("cc","xiaohong"));
    let a = init_arrays();
    for find in a.iter(){
       let c = match users.binary_search_by(|u| u.id.cmp(&find)) {
           Ok(t)=>t+1,
           _=>0usize
       } ;
        info!("find:{:?}",c);
    }

}

fn init_arrays()->Vec<ArrayString<[u8; 63]>>{
    let mut a:Vec<ArrayString<[u8; 63]>> = Vec::new();
    a.push(init_array("g1"));
    a.push(init_array("h2"));
    a.push(init_array("q3"));
    info!("{:?}",&a);
    a
}

fn init_array(value: &str)->ArrayString<[u8; 63]>{
    let mut a:ArrayString<[u8; 63]> = ArrayString::new();
    a.push_str(value);
    info!("{:?}",&a);
    a
}

fn init_user (id:&str,name :&str) ->User{
    User{
        name:name.into(),
        id:init_array(id)
    }
}
