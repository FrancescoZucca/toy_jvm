#![allow(dead_code)]

extern crate core;

use std::borrow::BorrowMut;
use std::fs::{File};
use std::collections::HashMap;
use num_traits::FromPrimitive;
use opcodes::Opcodes::*;
use crate::types::{ArrayTypes, Attribute, Const, ConstPool, Field, MethodAccessFlags, Types};
use crate::loader::Loader;
use crate::opcodes::Opcodes;
use crate::Types::*;

mod opcodes;
mod types;
pub mod loader;
mod natives;

static mut L: Loader = Loader{r: None, loaded_classes: None};


pub struct Frame<'a>{
    class: &'a mut Class,
    ip: u32,
    code: Vec<u8>,
    locals: Vec<Types>,
    stack: Vec<Types>,
    arrays: Vec<Vec<Types>>,
    native: bool,
    native_fn: Option<&'a fn(&crate::Class, Vec<Types>) -> Types>
}

#[derive(Debug, Clone)]
pub struct Class{
    cp: ConstPool,
    name: String,
    supr: String,
    flags: u16,
    interfaces: Vec<String>,
    fields: Vec<Field>,
    methods: Vec<Field>,
    attributes: Vec<Attribute>,
    version: [u16; 2]
}

impl Class{
    pub fn frame(&mut self, method: String, desc: String, args: Vec<Types>) -> Frame{
        println!("Loading method {}::{} with locals {:?}",self.name, method, args);
        for m in &self.methods{
            if m.name == method && m.desc == desc{
                if MethodAccessFlags::new(m.flags).NATIVE{
                    println!("Loading native...");
                    unsafe {
                        if let Some(n) = &mut natives::NATIVES {
                            return Frame {
                                native_fn: Some(n.get(&(self.name.clone(), m.name.clone(), m.desc.clone())).unwrap()),
                                class: self,
                                ip: 0,
                                code: vec![],
                                locals: args,
                                stack: vec![],
                                arrays: vec![],
                                native: true,
                            }
                        }
                    }
                }
                for a in &m.attr{
                    if a.name == "Code" && a.data.len() > 8{
                        let max_locals = u16::from_be_bytes([a.data[2],a.data[3]]);
                        println!("max locals: {}", max_locals);
                        let mut frame = Frame{
                            ip: 0,
                            code: a.data[8..].to_vec(),
                            locals: Vec::with_capacity(max_locals as usize),
                            stack: Vec::new(),
                            class: self,
                            arrays: Vec::new(),
                            native: false,
                            native_fn: None
                        };
                        frame.locals.resize(max_locals.into(), Int(0));
                        for (i, item) in args.iter().enumerate(){
                            frame.locals[i] = item.clone();
                        }
                        return frame;
                    }
                }
            }
        }
        panic!("Method {}:{} not found in class {}!", method, desc, self.name);
    }
}

impl Frame<'_>{
    pub fn pop(&mut self) -> Types{
        return self.stack.pop().expect("Stack empty");
    }

    pub fn exec(&mut self) -> Types{

        if self.native{
            return self.native_fn.unwrap()(self.class, self.locals.clone());
        }

        loop{
            let op: Opcodes = FromPrimitive::from_u8(self.code[self.ip as usize]).expect("Invalid opcode.");
            println!{"Executing opcode {:?} ({}) with stack {:?}", op, op as u8, self.stack};

            match op{
                NOP => {}
                ICONST_M1 => self.stack.push(Int(-1)),
                ICONST_0 => self.stack.push(Int(0)),
                ICONST_1 => self.stack.push(Int(1)),
                ICONST_2 => self.stack.push(Int(2)),
                ICONST_3 => self.stack.push(Int(3)),
                ICONST_4 => self.stack.push(Int(4)),
                ICONST_5 => self.stack.push(Int(5)),
                LCONST_0 => self.stack.push(Long(0)),
                LCONST_1 => self.stack.push(Long(1)),
                BIPUSH => {
                    self.ip = self.ip +1;
                    let val = self.code[self.ip as usize];
                    self.stack.push(Int(val as i32))
                },
                ILOAD_0 | DLOAD_0 | ALOAD_0 => self.stack.push(self.locals[0].clone()),
                ILOAD_1 | DLOAD_1 | ALOAD_1 => self.stack.push(self.locals[1].clone()),
                ILOAD_2 | DLOAD_2 | ALOAD_2 => self.stack.push(self.locals[2].clone()),
                ILOAD_3 | DLOAD_3 | ALOAD_3 => self.stack.push(self.locals[3].clone()),
                ASTORE_0 | ISTORE_0 => self.locals[0] = self.pop(),
                ASTORE_1 | ISTORE_1 => self.locals[1] = self.pop(),
                ASTORE_2 | ISTORE_2 => self.locals[2] = self.pop(),
                ASTORE_3 | ISTORE_3 => self.locals[3] = self.pop(),
                IASTORE => {
                    let val = self.pop();
                    let idx = self.pop_int() as usize;
                    let array_ref = self.pop();
                    if let Array((a_idx, typ)) = array_ref{
                        self.arrays[a_idx][idx] = val;
                    }else{
                        panic!("Invalid Array.");
                    }
                },
                IALOAD => {
                    let idx = self.pop_int() as usize;
                    let array_ref = self.pop();

                    if let Array((a_idx, typ)) = array_ref{
                        self.stack.push(self.arrays[a_idx][idx].clone());
                    }else{
                        panic!("Invalid array.");
                    }
                },
                POP => {self.stack.pop();},
                POP2 => {
                    let val = self.pop();
                    match val{
                        Long(_) | Double(_) => {},
                        _ => {self.pop();},
                    }
                },
                DUP => {
                    let val = self.pop();
                    self.stack.push(val.clone());
                    self.stack.push(val);
                },
                IADD => {
                    let b = self.pop_int();
                    let a = self.pop_int();
                    self.stack.push(Int(a+b));
                },
                ISUB => {
                    let a = self.pop_int();
                    let b = self.pop_int();
                    self.stack.push(Int(b-a))
                },
                DADD => {
                    let a = self.pop_double();
                    let b = self.pop_double();
                    self.stack.push(Double(a+b));
                },
                DSUB => {
                    let a = self.pop_double();
                    let b = self.pop_double();
                    self.stack.push(Double(b-a));
                },
                LDC => unsafe{
                    let idx = self.code[self.ip as usize + 1];
                    self.ip += 1;

                    match self.class.cp.get(idx as u16) {
                        Const::Str(s) => { self.stack.push(Str(s)) }
                        Const::Int(i) => { self.stack.push(Int(i)) }
                        Const::Float(f) => { self.stack.push(Float(f)) }
                        Const::Class(name_idx) => {
                            let class = L.get_class(L.resolve(&self.class.cp, name_idx as usize));
                            self.stack.push(Class(class.name.clone()));
                        }
                        Const::FMIRef(fmi) => {
                            let (clname, name, desc) = self.handle_fmi(Const::FMIRef(fmi));
                            unimplemented!()
                        }
                        _ => panic!()
                    }
                }
                LDC2_W => {
                    let idx = u16::from_be_bytes(self.read_bytes());
                    let val = &self.class.cp.consts[idx as usize - 1];
                    println!("{:?}", val);
                    if let Const::Double(i) = val{
                        self.stack.push(Double(*i))
                    }else if let Const::Long(l) = val{
                        self.stack.push(Long(*l))
                    }else{
                        println!("Called LDC2_W on a NON-LONG! Ignoring...");
                    }
                },
                IFGE => {
                    let offset = u16::from_be_bytes(self.read_bytes());
                    let a = self.pop_int();
                    if a >= 0{
                        self.ip += offset as u32 - 1;
                    }else{
                        self.ip += 2;
                    }
                },
                IF_ICMPNE => {
                    let offset = u16::from_be_bytes(self.read_bytes());
                    let a = self.pop_int();
                    let b = self.pop_int();
                    if a != b{
                        self.ip += offset as u32 - 1;
                    }else{
                        self.ip += 2;
                    }
                },
                IRETURN | DRETURN | LRETURN | ARETURN | FRETURN => return self.pop(),
                RETURN => return Void,
                GETFIELD => unsafe {
                    let idx = u16::from_be_bytes(self.read_bytes());

                    let field = self.class.cp.get(idx);
                    let (clname, fname, fdesc) = self.handle_fmi(field);
                    let cl = L.get_class(clname.clone());
                    println!("{:?}", cl.fields);for field in &mut cl.fields{
                        if field.name == fname && field.desc == fdesc{
                            println!("{}::{}-{} = {:?}", clname, fname, fdesc, field.value.clone());
                            self.stack.push(field.value.borrow_mut().clone().unwrap());
                            break;
                        }
                    }
                }
                PUTFIELD => unsafe{
                    let idx = u16::from_be_bytes([self.code[self.ip as usize+1], self.code[self.ip as usize+2]]);
                    self.ip = self.ip + 2;

                    let field = self.class.cp.get(idx);
                    if let Const::FMIRef((class_idx, nat_idx)) = field{
                        let nat = self.class.cp.get(nat_idx);
                        if let Const::NameAndType((name_idx, typ_idx)) = nat {
                            if let Const::Class(clname_idx) = self.class.cp.get(class_idx){
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
                GETSTATIC => unsafe{ //TODO: Properly initialise values.
                    let idx = u16::from_be_bytes(self.read_bytes());

                    let field_ref = self.class.cp.get(idx);
                    let (clname, fname, ftype) = self.handle_fmi(field_ref);
                    println!("{}: {}-{}", clname, fname, ftype);
                    let class = L.get_class(clname);
                    for field in &class.fields{
                        if field.name == fname && field.desc == ftype{
                            println!("{}, {}-{} : {:?}", self.class.name, field.name, field.desc, field.value);
                            self.stack.push(field.clone().value.unwrap_or(Void));
                            break;

                        }
                    }
                },
                PUTSTATIC => unsafe{
                    let idx = u16::from_be_bytes(self.read_bytes());
                    let (clname, fname, ftype) = self.handle_fmi(self.class.cp.get(idx));
                    let cl = L.get_class(clname.clone());
                    for field in &mut cl.fields{
                        if field.name == fname && field.desc == ftype{
                            let val = self.pop();
                            println!("{}::{} set to {:?}", clname, fname, val);
                            field.value = Some(val);
                            break;
                        }
                    }
                },
                INVOKEVIRTUAL => unsafe {
                    let idx = u16::from_be_bytes([self.code[self.ip as usize+1], self.code[self.ip as usize+2]]);
                    self.ip = self.ip + 2;

                    let method = self.class.cp.get(idx);
                    let (clname, mname, typ) = self.handle_fmi(method);
                    println!("Resolving class {}..", clname);
                    let c = L.get_class(clname);

                    let mut v: Vec<Types> = Vec::new();
                    for ch in typ.chars(){
                        match ch{
                            'I' => v.push(Int(self.pop_int())),
                            ')' => break,
                            _ => {}
                        }
                    }
                    v.push(self.pop());
                    v.reverse();
                    let mut frame = c.frame(mname, typ, v);
                    match frame.exec() {
                        Void => {},
                        val => self.stack.push(val)
                    }
                },
                INVOKESPECIAL => unsafe {
                    let idx = u16::from_be_bytes([self.code[self.ip as usize+1], self.code[self.ip as usize+2]]);
                    self.ip = self.ip + 2;

                    let method = self.class.cp.get(idx);
                    if let Const::FMIRef((class_idx, nat_idx)) = method{
                        let nat = self.class.cp.get(nat_idx);
                        if let Const::NameAndType((name_idx, typ_idx)) = nat {
                            if let Const::Class(clname_idx) = self.class.cp.get(class_idx){

                                let clname = L.resolve(&mut self.class.cp, clname_idx as usize);
                                println!("Resolving class {}..", clname);
                                let c = L.get_class(clname);

                                let mut v: Vec<Types> = Vec::new();
                                let typ = L.resolve(&mut self.class.cp, typ_idx as usize);
                                for ch in typ.chars(){
                                    match ch{
                                        'I' => v.push(Int(self.pop_int())),
                                        ')' => break,
                                        _ => {}
                                    }
                                }
                                v.push(self.pop());
                                v.reverse();
                                let mut frame = c.frame(L.resolve(&mut self.class.cp, name_idx as usize), typ, v);
                                match frame.exec() {
                                    Void => {},
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
                INVOKESTATIC => unsafe {
                    let idx = u16::from_be_bytes([self.code[self.ip as usize+1], self.code[self.ip as usize+2]]);
                    self.ip = self.ip + 2;

                    let method = self.class.cp.get(idx);
                    if let Const::FMIRef((class_idx, nat_idx)) = method{
                        let nat = self.class.cp.get(nat_idx);
                        if let Const::NameAndType((name_idx, typ_idx)) = nat {
                            if let Const::Class(clname_idx) = self.class.cp.get(class_idx){
                                //println!("{}, {}, {}", name_idx, typ_idx, clname_idx);
                                //println!("{} {}, {}", self.class.cp.resolve(name), self.class.cp.resolve(typ), self.class.cp.resolve(clname));

                                let clname = L.resolve(&mut self.class.cp, clname_idx as usize);
                                let c = L.get_class(clname);

                                let mut v: Vec<Types> = Vec::new();
                                let typ = L.resolve(&mut self.class.cp, typ_idx as usize);
                                for ch in typ.chars(){
                                    match ch{
                                        'I' => v.push(Int(self.pop_int())),
                                        'D' => { v.push(Void);v.push(self.pop());},
                                        ')' => break,
                                        _ => {}
                                    }
                                }
                                v.reverse();
                                let mut frame = c.frame(L.resolve(&mut self.class.cp, name_idx as usize), typ,v);
                                match frame.exec() {
                                    Void => {},
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
                NEW => unsafe {
                    let idx = u16::from_be_bytes([self.code[self.ip as usize+1], self.code[self.ip as usize+2]]);
                    self.ip = self.ip + 2;

                    println!("...test");
                    if let Const::Class(class_idx) = self.class.cp.get(idx) {
                        let class = L.get_class(L.resolve(&mut self.class.cp, class_idx as usize));
                        self.stack.push(Types::Class(class.name.clone()))
                    }else{
                        panic!("Tried instantiating a non-class");
                    }
                },
                NEWARRAY => {
                    let typ: ArrayTypes = FromPrimitive::from_u8(self.code[self.ip as usize + 1]).unwrap();
                    self.ip += 1;

                    let count = self.pop_int();
                    let mut v = Vec::<Types>::with_capacity(count as usize);
                    v.resize(count as usize, Void);

                    self.arrays.push(v);
                    let idx = self.arrays.len() - 1;

                    self.stack.push(Array((idx, typ)));
                }
                opc => panic!("Unimplemented opcode {:?}", opc)
            }
            self.ip = self.ip + 1;
        }
    }

    fn pop_int(&mut self) -> i32{
        return if let Int(i) = self.stack.pop().expect("Stack empty"){i}else{panic!("Expected i32 on the stack.")};
    }

    fn pop_double(&mut self) -> f64{
        return if let Double(f) = self.pop(){f}else{panic!("Expected double on the stack")}
    }

    fn handle_fmi(&self, fmi_ref: Const) -> (String, String, String){
        if let Const::FMIRef((class_idx, nat_idx)) = fmi_ref{
            let nat = self.class.cp.get(nat_idx);
            if let Const::NameAndType((name_idx, type_idx)) = nat{
                let class = self.class.cp.get(class_idx);
                if let Const::Class(clname_idx) = class {
                    unsafe{
                        let clname = L.resolve(&self.class.cp, clname_idx as usize);
                        let name = L.resolve(&self.class.cp, name_idx as usize);
                        let typ = L.resolve(&self.class.cp, type_idx as usize);

                        return (clname, name, typ);
                    }
                }else{
                    panic!("Expected Class.");
                }
            }else{
                panic!("Expected NAT.");
            }
        }else{
            panic!("Expected FMI ref.");
        }
    }

    fn read_bytes<const T: usize>(&mut self) -> [u8; T]{
        let mut r = [0u8; T];
        for i in 0..T{
            r[i] = self.code[self.ip as usize + 1];
            self.ip += 1;
        }
        return r;
    }
}

fn main() -> std::io::Result<()> {
    unsafe{
        L = Loader{
            r: Some(File::open("Add.class").unwrap()),
            loaded_classes: Some(HashMap::new())
        }
    }

    natives::load_natives();

    let clname = unsafe { L.load_class(None)};
    let c = unsafe{L.get_class(clname)};

    let mut frame = c.frame("main".to_string(), "([Ljava/lang/String;)V".to_string() ,vec!());
    frame.exec();

    Ok(())
}
