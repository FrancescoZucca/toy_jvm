use std::collections::HashMap;
use crate::Types;
use Types::*;

pub static mut NATIVES: Option<HashMap<(String, String, String), fn(&crate::Class, Vec<Types>) -> Types>> = None;

pub fn load_natives(){
    unsafe{
        NATIVES = Some(HashMap::new());
        if let Some(n) = &mut NATIVES{
            n.insert(("java/lang/System".to_string() ,"registerNatives".to_string(), "()V".to_string()), register_natives);
        }
    }
}

pub fn register_natives(c: &crate::Class, _: Vec<Types>) -> Types{
    for f in c.fields.clone(){
        println!("{}", f.name)
    }
    Void
}