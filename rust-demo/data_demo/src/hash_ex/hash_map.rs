use std::collections::HashMap;

pub fn hash_main() {
    let mut a = HashMap::new();
    let mut b = HashMap::new();
    let user1 = User {
        name: String::from("xiaohong"),
        age: 18,
    };
    a.insert(String::from("person1"), user1);
    b.insert(String::from("people"), a);

    let c = b.clone();
    let map1 = get_map_usr(c, String::from("people1"));
    info!("{:?}", map1);
    let d = b.clone();
    let map2 = get_map_usr(d, String::from("people"));
    info!("{:?}", map2);

    map_iter(map2);
}

#[derive(Debug, Clone)]
pub struct User {
    name: String,
    age: i32,
}

pub fn map_iter(map: HashMap<String, User>) {
    for (key, value) in map.iter() {
        info!("key:{:?},value:{:?}",key,value)
    }
}

pub fn get_map_usr(map: HashMap<String, HashMap<String, User>>, key: String) -> HashMap<String, User> {
//    let b = map.clone();
    let c = map.get(key.as_str());
    let map = match c {
        Some(t) => t.clone(),
        _ => {
            let a: HashMap<String, User> = HashMap::new();
            a
        }
    };
    map
}
