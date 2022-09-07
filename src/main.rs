#![allow(dead_code)]

use std::borrow::BorrowMut;
use std::io::Read;
use std::fs::{File};
use crate::ConstTypes::*;
use std::collections::HashMap;
use std::path::Path;

static mut L: Loader = Loader{r: None, loaded_classes: None};

struct Loader{
    r: Option<File>,
    loaded_classes: Option<HashMap<String, Class>>
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

    fn resolve(&mut self, cp: &mut ConstPool, i: usize) -> String{
        return match &cp.consts[i - 1].data {
            Str(s) => s.clone(),
            Class(idx) => self.resolve(cp, *idx as usize),
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
        for _ in 1..cp_count {
            let tag = self.u1();
            let c: Const;
            match tag{
                0x01 => c = Const{tag, data:Str(String::from_utf8(self.vec_bytes_u2()).unwrap())},
                0x03 => c = Const{tag, data:Int(i32::from_be_bytes(self.bytes()))},
                0x04 => c = Const{tag, data:Float(f32::from_be_bytes(self.bytes()))},
                0x07 => c = Const{tag, data:Class(self.u2())},
                0x08 => c = Const{tag, data:StrIndex(self.u2())},
                0x09|0xa => c = Const{tag, data:FMIRef((self.u2(), self.u2()))},
                0x0c => c = Const{tag, data:NameAndType((self.u2(), self.u2()))},
                _all => panic!("Error parsing tag {}: Not implemented!", _all)
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

#[derive(Debug, Clone)]
struct Const{
    tag: u8,
    data: ConstTypes
}

#[derive(Debug, Clone)]
struct Field{
    flags: u16,
    name: String,
    desc: String,
    attr: Vec<Attribute>,
    value: Option<Types>
}

#[derive(Debug, Clone)]
struct Attribute{
    name: String,
    data: Vec<u8>
}

#[derive(Debug, Clone)]
enum ConstTypes{
    Str(String),
    Int(i32),
    Float(f32),
    Class(u16),
    FMIRef((u16, u16)),
    StrIndex(u16),
    NameAndType((u16, u16)),
}

#[derive(Debug, Clone)]
enum Types{
    Int(i32),
    Void,
    Class(String)
}

#[derive(Debug, Clone)]
struct ConstPool{
    consts: Vec<Const>
}

impl ConstPool{
    pub fn get(&self, idx:u16) -> Const{
        let idx = (idx-1) as usize;
        return self.consts[idx].clone();
    }
}

struct Frame<'a>{
    class: &'a mut Class,
    ip: u32,
    code: Vec<u8>,
    locals: Vec<Types>,
    stack: Vec<Types>
}

#[derive(Debug, Clone)]
struct Class{
    cp: ConstPool,
    name: String,
    supr: String,
    flags: u16,
    interfaces: Vec<String>,
    fields: Vec<Field>,
    methods: Vec<Field>,
    attributes: Vec<Attribute>
}

impl Class{
    pub fn frame(&mut self, method: String, args: Vec<Types>) -> Frame{
        println!("Loading method {} with args {:?}", method, args);
        for m in &self.methods{
            if m.name == method{
                for a in &m.attr{
                    if a.name == "Code" && a.data.len() > 8{
                        let max_locals = u16::from_be_bytes([a.data[2],a.data[3]]);
                        println!("max locals: {}", max_locals);
                        let mut frame = Frame{
                            ip: 0,
                            code: a.data[8..].to_vec(),
                            locals: Vec::with_capacity(max_locals as usize),
                            stack: Vec::new(),
                            class: self
                        };
                        frame.locals.resize(max_locals.into(), Types::Int(0));
                        for (i, item) in args.iter().enumerate(){
                            frame.locals[i] = item.clone();
                        }
                        return frame;
                    }
                }
            }
        }
        panic!("Method not found!");
    }
}

impl Frame<'_>{
    pub fn pop(&mut self) -> Types{
        return self.stack.pop().expect("Stack empty");
    }

    pub fn exec(&mut self) -> Types{
        loop{
            let op = self.code[self.ip as usize];
            println!{"Executing opcode {} with stack {:?}", op, self.stack};

            match op{
                2 => self.stack.push(Types::Int(-1)),
                3 => self.stack.push(Types::Int(0)),
                4 => self.stack.push(Types::Int(1)),
                5 => self.stack.push(Types::Int(2)),
                6 => self.stack.push(Types::Int(3)),
                7 => self.stack.push(Types::Int(4)),
                8 => self.stack.push(Types::Int(5)),
                16 => {
                    self.ip = self.ip +1;
                    let val = self.code[self.ip as usize];
                    self.stack.push(Types::Int(val as i32))
                }
                26 => self.stack.push(self.locals[0].clone()),
                27 => self.stack.push(self.locals[1].clone()),
                42 => self.stack.push(self.locals[0].clone()),
                43 => self.stack.push(self.locals[1].clone()),
                44 => self.stack.push(self.locals[2].clone()),
                45 => self.stack.push(self.locals[3].clone()),
                75 => self.locals[0] = self.pop(),
                76 => self.locals[1] = self.pop(),
                77 => self.locals[2] = self.pop(),
                78 => self.locals[3] = self.pop(),
                87 => {self.stack.pop();},
                89 => {
                    let val = self.pop();
                    self.stack.push(val.clone());
                    self.stack.push(val);
                },
                96 => {
                      let b = self.pop_int();
                    let a = self.pop_int();
                    self.stack.push(Types::Int(a+b));
                },
                100 => {
                    let a = self.pop_int();
                    let b = self.pop_int();
                    self.stack.push(Types::Int(a-b))
                },
                172 => return self.pop(),
                177 => return Types::Void,
                180 => unsafe {
                    let idx = u16::from_be_bytes([self.code[self.ip as usize+1], self.code[self.ip as usize+2]]);
                    self.ip = self.ip + 2;

                    let field = self.class.cp.get(idx);
                    if let FMIRef((class_idx, nat_idx)) = field.data{
                        let nat = self.class.cp.get(nat_idx);
                        if let NameAndType((name_idx, typ_idx)) = nat.data {
                            if let Class(clname_idx) = self.class.cp.get(class_idx).data{
                                let c = L.get_class(L.resolve(&mut self.class.cp, clname_idx as usize));
                                let value = self.pop();
                                if let Types::Class(name) = value{
                                    let cl = L.get_class(name);
                                    println!("{:?}", cl.fields);
                                    let fname = L.resolve(&mut self.class.cp, name_idx as usize);
                                    let fdesc = L.resolve(&mut self.class.cp, typ_idx as usize);
                                    for field in &mut cl.fields{
                                        if field.name == fname && field.desc == fdesc{
                                            self.stack.push(field.value.borrow_mut().clone().unwrap());
                                            break;
                                        }
                                    }
                                }
                            }else{
                                panic!("Corrupted Class");
                            }
                        }else{
                            panic!("Corrupted Method Constant.");
                        }
                    }else{
                        panic!("Tried to invoke a non-method constant");
                    }
                }
                181 => unsafe{
                    let idx = u16::from_be_bytes([self.code[self.ip as usize+1], self.code[self.ip as usize+2]]);
                    self.ip = self.ip + 2;

                    let field = self.class.cp.get(idx);
                    if let FMIRef((class_idx, nat_idx)) = field.data{
                        let nat = self.class.cp.get(nat_idx);
                        if let NameAndType((name_idx, typ_idx)) = nat.data {
                            if let Class(clname_idx) = self.class.cp.get(class_idx).data{
                                let c = L.get_class(L.resolve(&mut self.class.cp, clname_idx as usize));
                                let value = self.pop();
                                if let Types::Class(name) = self.pop(){
                                    let cl = L.get_class(name);
                                    for field in &mut cl.fields{
                                        if field.name == L.resolve(&mut self.class.cp, name_idx as usize) && field.desc == L.resolve(&mut self.class.cp, typ_idx as usize){
                                            field.value = Some(value);
                                            break;
                                        }
                                    }
                                }
                            }else{
                                panic!("Corrupted Class");
                            }
                        }else{
                            panic!("Corrupted Method Constant.");
                        }
                    }else{
                        panic!("Tried to invoke a non-method constant");
                    }
                },
                182 => unsafe {
                    let idx = u16::from_be_bytes([self.code[self.ip as usize+1], self.code[self.ip as usize+2]]);
                    self.ip = self.ip + 2;

                    let method = self.class.cp.get(idx);
                    if let FMIRef((class_idx, nat_idx)) = method.data{
                        let nat = self.class.cp.get(nat_idx);
                        if let NameAndType((name_idx, typ_idx)) = nat.data {
                            if let Class(clname_idx) = self.class.cp.get(class_idx).data{

                                let clname = L.resolve(&mut self.class.cp, clname_idx as usize);
                                println!("Resolving class {}..", clname);
                                let c = L.get_class(clname);

                                let mut v: Vec<Types> = Vec::new();
                                let typ = L.resolve(&mut self.class.cp, typ_idx as usize);
                                for ch in typ.chars(){
                                    match ch{
                                        'I' => v.push(Types::Int(self.pop_int())),
                                        ')' => break,
                                        _ => {}
                                    }
                                }
                                v.push(self.pop());
                                v.reverse();
                                let mut frame = c.frame(L.resolve(&mut self.class.cp, name_idx as usize), v);
                                match frame.exec() {
                                    Types::Void => {},
                                    val => self.stack.push(val)
                                }
                            }else{
                                panic!("Corrupted Class");
                            }
                        }else{
                            panic!("Corrupted Method Constant.");
                        }
                    }else{
                        panic!("Tried to invoke a non-method constant");
                    }
                },
                183 => unsafe {
                    let idx = u16::from_be_bytes([self.code[self.ip as usize+1], self.code[self.ip as usize+2]]);
                    self.ip = self.ip + 2;

                    let method = self.class.cp.get(idx);
                    if let FMIRef((class_idx, nat_idx)) = method.data{
                        let nat = self.class.cp.get(nat_idx);
                        if let NameAndType((name_idx, typ_idx)) = nat.data {
                            if let Class(clname_idx) = self.class.cp.get(class_idx).data{

                                let clname = L.resolve(&mut self.class.cp, clname_idx as usize);
                                println!("Resolving class {}..", clname);
                                let c = L.get_class(clname);

                                let mut v: Vec<Types> = Vec::new();
                                let typ = L.resolve(&mut self.class.cp, typ_idx as usize);
                                for ch in typ.chars(){
                                    match ch{
                                        'I' => v.push(Types::Int(self.pop_int())),
                                        ')' => break,
                                        _ => {}
                                    }
                                }
                                v.push(self.pop());
                                v.reverse();
                                let mut frame = c.frame(L.resolve(&mut self.class.cp, name_idx as usize), v);
                                match frame.exec() {
                                    Types::Void => {},
                                    val => self.stack.push(val)
                                }
                            }else{
                                panic!("Corrupted Class");
                            }
                        }else{
                            panic!("Corrupted Method Constant.");
                        }
                    }else{
                        panic!("Tried to invoke a non-method constant");
                    }
                },
                184 => unsafe {
                    let idx = u16::from_be_bytes([self.code[self.ip as usize+1], self.code[self.ip as usize+2]]);
                    self.ip = self.ip + 2;

                    let method = self.class.cp.get(idx);
                    if let FMIRef((class_idx, nat_idx)) = method.data{
                        let nat = self.class.cp.get(nat_idx);
                        if let NameAndType((name_idx, typ_idx)) = nat.data {
                            if let Class(clname_idx) = self.class.cp.get(class_idx).data{
                                //println!("{}, {}, {}", name_idx, typ_idx, clname_idx);
                                //println!("{} {}, {}", self.class.cp.resolve(name), self.class.cp.resolve(typ), self.class.cp.resolve(clname));

                                let clname = L.resolve(&mut self.class.cp, clname_idx as usize);
                                let c = L.get_class(clname);

                                let mut v: Vec<Types> = Vec::new(); 
                                let typ = L.resolve(&mut self.class.cp, typ_idx as usize);
                                for ch in typ.chars(){
                                    match ch{
                                        'I' => v.push(Types::Int(self.pop_int())),
                                        ')' => break,
                                        _ => {}
                                    }
                                }
                                v.reverse();
                                let mut frame = c.frame(L.resolve(&mut self.class.cp, name_idx as usize), v);
                                match frame.exec() {
                                    Types::Void => {},
                                    val => self.stack.push(val)
                                }
                            }else{
                                panic!("Corrupted Class");
                            }
                        }else{
                            panic!("Corrupted Method Constant.");
                        }
                    }else{
                        panic!("Tried to invoke a non-method constant");
                    }
                },
                187 => unsafe {
                    let idx = u16::from_be_bytes([self.code[self.ip as usize+1], self.code[self.ip as usize+2]]);
                    self.ip = self.ip + 2;

                    if let Class(class_idx) = self.class.cp.get(idx).data {
                        let class = L.get_class(L.resolve(&mut self.class.cp, class_idx as usize));
                        self.stack.push(Types::Class(class.name.clone()))
                    }else{
                        panic!("Tried instantiating a non-class");
                    }
                },
                opc => panic!("Unimplemented opcode {}", opc)
            }
            self.ip = self.ip + 1;
        }
    }

    fn pop_int(&mut self) -> i32{
        return if let Types::Int(i) = self.stack.pop().expect("Stack empty"){i}else{panic!("Expected i32 on the stack.")};
    }
}

fn main() -> std::io::Result<()> {
    unsafe{
        L = Loader{
            r: Some(File::open("Add.class").unwrap()),
            loaded_classes: Some(HashMap::new())
        }
    }

    let clname = unsafe { L.load_class(None)};
    let c = unsafe{L.get_class(clname)};

    let mut frame = c.frame("main".to_string(), vec!());
    frame.exec();

    Ok(())
}
