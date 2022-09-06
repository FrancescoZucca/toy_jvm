#![allow(dead_code)]

use std::io::Read;
use std::fs::File;
use crate::ConstTypes::*;
use std::collections::HashMap;

struct Loader<T: Read> {
    r: T
}

impl<T: Read> Loader<T>{
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
        self.r.read_exact(&mut buf[..]).unwrap();
        return buf;
    }

    pub fn vec_bytes(&mut self, n: usize) -> Vec<u8>{
        let mut vec = Vec::with_capacity(n);
        vec.resize(n, 0);
        self.r.read_exact(&mut vec).unwrap();
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
        match &cp.consts[i - 1].data{
            Str(s) => return s.clone(),
            _ => return String::new()
        }
    }

    fn resolve_u2(&mut self, cp: &mut ConstPool) -> String{
        let idx = self.u2() as usize;
        return self.resolve(cp, idx);
    }

    fn cpinfo(&mut self) -> ConstPool{
        let mut cp = ConstPool{consts: Vec::new()};
        let cp_count = self.u2();
        for _ in 1..cp_count {
            let tag = self.u1();
            let mut c: Const;
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
                attr: self.attributes(cp)
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

    pub fn load_class(&mut self, hm: &mut HashMap<String, Class>) -> String{
        assert_eq!(0xcafebabeu32, self.u4());
        println!("Java version: {}.{}", self.u2(), self.u2());

        let mut cp = self.cpinfo();
        let flags = self.u2();
        let name = self.resolve_u2(&mut cp);
        let supr = self.resolve_u2(&mut cp);
        let interfaces = self.interfaces(&mut cp);
        let fields = self.fields(&mut cp);
        let methods = self.fields(&mut cp);
        let attributes = self. attributes(&mut cp);

        let c = Class{
            cp,
            flags,
            name,
            supr,
            interfaces,
            fields,
            methods,
            attributes
        };

        hm.insert(name, c);
        return name;
    }
}

#[derive(Debug, Clone)]
struct Const{
    tag: u8,
    data: ConstTypes
}

struct Field{
    flags: u16,
    name: String,
    desc: String,
    attr: Vec<Attribute>
}

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
    Int(i32)
}

struct ConstPool{
    consts: Vec<Const>
}

impl ConstPool{
    pub fn resolve(&self, idx: u16) -> String{
        let idx = idx as usize;
        if let Str(s) = &self.consts[idx-1].data {
            return s.clone();
        }else{ return String::from("");}
    }

    pub fn get(&self, idx:u16) -> Const{
        let idx = (idx-1) as usize;
        return self.consts[idx].clone();
    }
}

struct Frame<'a>{
    class: &'a mut Class,
    IP: u32,
    code: Vec<u8>,
    locals: Vec<Types>,
    stack: Vec<Types>
}

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
                            IP: 0,
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
    pub fn exec(&mut self, mut lc: HashMap<String, Class>) -> Types{
        loop{
            let op = self.code[self.IP as usize];
            println!{"Executing opcode {} with stack {:?}", op, self.stack};

            match op{
                2 => self.stack.push(Types::Int(-1)),
                3 => self.stack.push(Types::Int(0)),
                4 => self.stack.push(Types::Int(1)),
                5 => self.stack.push(Types::Int(2)),
                6 => self.stack.push(Types::Int(3)),
                7 => self.stack.push(Types::Int(4)),
                8 => self.stack.push(Types::Int(5)),
                26 => self.stack.push(self.locals[0].clone()),
                27 => self.stack.push(self.locals[1].clone()),
                96 => {
                      let a = self.pop_int();
                    let b = self.pop_int();
                    self.stack.push(Types::Int(a+b));
                },
                172 => return self.stack.pop().expect("Stack empty."),
                184 => {
                    let mut idx = u16::from_be_bytes([self.code[self.IP as usize+1], self.code[self.IP as usize+2]]);
                    self.IP = self.IP + 2;

                    let method = self.class.cp.get(idx);
                    if let FMIRef((class_idx, nat_idx)) = method.data{
                        let nat = self.class.cp.get(nat_idx);
                        if let NameAndType((name_idx, typ_idx)) = nat.data {
                            if let Class(clname_idx) = self.class.cp.get(class_idx).data{
                                //println!("{}, {}, {}", name_idx, typ_idx, clname_idx);
                                //println!("{} {}, {}", self.class.cp.resolve(name), self.class.cp.resolve(typ), self.class.cp.resolve(clname));

                                let clname = self.class.cp.resolve(clname_idx);

                                let c = match lc.get_mut(&clname){
                                    Some(c) => {
                                        c
                                    },
                                    None => {
                                        panic!();
                                    }
                                };

                                let mut v: Vec<Types> = Vec::new(); 
                                let typ = self.class.cp.resolve(typ_idx);
                                for ch in typ.chars(){
                                    match ch{
                                        'I' => v.push(Types::Int(self.pop_int())),
                                        ')' => break
                                    }
                                }
                                v.reverse();
                                let mut frame = c.frame(self.class.cp.resolve(name_idx), v);
                                frame.exec(lc);
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
                opc => panic!("Unimplemented opcode {}", opc)
            }
            self.IP = self.IP + 1;
        }
    }

    fn pop_int(&mut self) -> i32{
        return if let Types::Int(i) = self.stack.pop().expect("Stack empty"){i}else{panic!("Expected i32 on the stack.")};
    }
}

fn main() -> std::io::Result<()> {
    let mut l = Loader{r: File::open("Add.class")?};

    let mut loaded_classes = HashMap::new();

    let clname = l.load_class(&mut loaded_classes);
    let mut class = loaded_classes.get_mut(&clname).expect("NoSuchClass");
    let mut frame = class.frame(String::from("main"), vec!());

    frame.exec(loaded_classes);

    Ok(())
}
