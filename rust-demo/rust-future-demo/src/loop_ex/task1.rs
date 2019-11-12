use futures::{Stream,Future};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::hash::Hash;
use futures::future::{ok};
use futures::future::{loop};
pub struct Controller {

}

#[derive(debug,Clone)]
pub struct Status{
    pub context:Mutex<Arc<HashMap<String,Config>>>
}

#[derive(debug,Clone)]
pub struct Config{
    pub name:String,
    pub age:i32
}


impl Status{
    pub fn get_context(&self)->Arc<HashMap<String,Config>>{
        let a = self.context.lock().unwrap();
        let a  = a.clone();
        a
    }

    pub fn chage_context(&mut self,key:String,value:Config){
        let a = self.context.lock().unwrap();

        let mut b:HashMap<String,Config> = HashMap::new();
        for (key1,value1)in a.iter(){
            b.insert(key1.clone(),value1.to_owned());
        }
        b.insert(key,value);
    }
}

impl Controller {
    pub fn new()->Contoller{

        Contoller{

        }
    }

    pub fn task(&self,status:Status)-> impl Future<Item=(), Error=()>{

        ok(())
    }
}

pub fn loop_task()-> impl Stream<Item=(Msg), Error=()>{
    loop{

    }
}