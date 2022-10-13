use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use crate::{Class, ConstTypes};
use crate::types::ConstTypes::*;
use crate::types::{Attribute, Const, ConstPool, Field};

pub struct Loader{
    pub(crate) r: Option<File>,
    pub(crate) loaded_classes: Option<HashMap<String, Class>>
}

impl Loader{
    pub fn u1(&mut self) -> u8{
        return self.bytes::<1>()[0];
    }

    pub fn u2(&mut self) -> u16 {
        return u16::from_be_bytes(self.bytes());
    }

    pub fn u4(&mut self) -> u32{
        return u32::from_be_bytes(self.bytes());
    }

    pub fn bytes<const N: usize>(&mut self) -> [u8; N]{
        let mut buf = [0 as u8; N];
        self.r.as_ref().unwrap().read_exact(&mut buf[..]).unwrap();
        return buf;
    }

    pub fn vec_bytes(&mut self, n: usize) -> Vec<u8>{
        let mut vec = Vec::with_capacity(n);
        vec.resize(n, 0);
        self.r.as_ref().unwrap().read_exact(&mut vec).unwrap();
        return vec;
    }

    pub fn vec_bytes_u4(&mut self) -> Vec<u8>{
        let num = self.u4() as usize;
        return self.vec_bytes(num);
    }

    pub fn vec_bytes_u2(&mut self) -> Vec<u8>{
        let num = self.u2() as usize;
        return self.vec_bytes(num);
    }

    pub(crate) fn resolve(&mut self, cp: &mut ConstPool, i: usize) -> String{
        return match &cp.consts[i - 1].data {
            ConstTypes::Str(s) => s.clone(),
            ConstTypes::Class(idx) => self.resolve(cp, *idx as usize),
            _ => {println!("Ritorno stringa vuota");String::new()}
        }
    }

    fn resolve_u2(&mut self, cp: &mut ConstPool) -> String{
        let idx = self.u2() as usize;
        return self.resolve(cp, idx);
    }

    fn resolve_super(&mut self, cp: &mut ConstPool) -> String{
        let idx = self.u2();
        return if idx == 0 {
            "".to_string()
        } else {
            self.resolve(cp, idx as usize)
        }
    }

    fn cpinfo(&mut self) -> ConstPool{
        let mut cp = ConstPool{consts: Vec::new()};
        let cp_count = self.u2();
        println!("Constants: {}", cp_count);
        let mut long_or_double = false;
        for _ in 1..cp_count {
            if long_or_double {
                long_or_double = false;
                cp.consts.push(Const{tag: 0, data:Invalid});
                continue;
            }
            let tag = self.u1();
            let c: Const;
            match tag{
                0x01 => c = Const{tag, data:ConstTypes::Str(String::from_utf8(self.vec_bytes_u2()).unwrap())},
                0x03 => c = Const{tag, data:ConstTypes::Int(i32::from_be_bytes(self.bytes()))},
                0x04 => c = Const{tag, data:ConstTypes::Float(f32::from_be_bytes(self.bytes()))},
                0x06 => c = { long_or_double = true; Const{tag, data:Double(f64::from_be_bytes(self.bytes()))}},
                0x07 => c = Const{tag, data:ConstTypes::Class(self.u2())},
                0x08 => c = Const{tag, data:ConstTypes::StrIndex(self.u2())},
                0x09|0xa => c = Const{tag, data:ConstTypes::FMIRef((self.u2(), self.u2()))},
                0x0c => c = Const{tag, data:ConstTypes::NameAndType((self.u2(), self.u2()))},
                _all => {println!("Error parsing tag {}: Not implemented!", _all); continue;}
            }
            cp.consts.push(c);

        }
        return cp;
    }

    fn interfaces(&mut self, cp: &mut ConstPool) -> Vec<String>{
        let mut v: Vec<String> = Vec::new();
        let interface_count = self.u2();
        for _ in 0..interface_count{
            v.push(self.resolve_u2(cp));
        }
        return v;
    }

    fn fields(&mut self, cp: &mut ConstPool) -> Vec<Field>{
        let mut v: Vec<Field> = Vec::new();
        let field_count = self.u2();
        for _ in 0..field_count{
            v.push(Field{
                flags: self.u2(),
                name: self.resolve_u2(cp),
                desc: self.resolve_u2(cp),
                attr: self.attributes(cp),
                value: None
            });
        }
        return v;
    }

    fn attributes(&mut self, cp: &mut ConstPool) -> Vec<Attribute>{
        let mut v: Vec<Attribute> = Vec::new();
        let attr_count = self.u2();

        for _ in 0..attr_count{
            v.push(Attribute{
                name: self.resolve_u2(cp),
                data: self.vec_bytes_u4()
            })
        }
        return v;
    }

    pub fn load_class(&mut self, f: Option<File>) -> String{

        if let Some(file) = f{
            self.r = Some(file);
        }

        assert_eq!(0xcafebabeu32, self.u4());
        println!("Java version: {}.{}", self.u2(), self.u2());

        let mut cp = self.cpinfo();
        let flags = self.u2();
        let name = self.resolve_u2(&mut cp);
        let supr = self.resolve_super(&mut cp);
        let interfaces = self.interfaces(&mut cp);
        let fields = self.fields(&mut cp);
        let methods = self.fields(&mut cp);
        let attributes = self. attributes(&mut cp);

        let c = Class{
            cp,
            flags,
            name: name.clone(),
            supr,
            interfaces,
            fields,
            methods,
            attributes
        };

        self.loaded_classes.as_mut().unwrap().insert(name.clone(), c);
        return name;
    }

    pub fn get_class(&mut self, name: String) -> &mut Class{

        let paths = vec!(Path::new("./"), Path::new("./src/"));

        let result = self.loaded_classes.as_ref().unwrap().get(&name);
        return match result {
            None => {
                let mut clname = String::new();
                for path in paths {
                    match File::open(path.join(Path::new(&(name.clone()+".class")))) {
                        Ok(file) => {clname = self.load_class(Some(file)); break;},
                        Err(_) => continue
                    }
                }
                if clname.is_empty(){
                    panic!("NoSuchMethod");
                }
                self.loaded_classes.as_mut().unwrap().get_mut(&clname).unwrap()
            }
            Some(_) => { self.loaded_classes.as_mut().unwrap().get_mut(&name).unwrap() }
        };
    }
}