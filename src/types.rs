use num_derive::FromPrimitive;

#[derive(FromPrimitive, Copy, Clone, Debug)]
pub enum ArrayTypes{
    BOOLEAN = 4,
    CHAR,
    FLOAT,
    DOUBLE,
    BYTE,
    SHORT,
    INT,
    LONG
}

#[derive(Debug, Clone)]
pub struct Field{
    pub flags: u16,
    pub name: String,
    pub desc: String,
    pub attr: Vec<Attribute>,
    pub value: Option<Types>
}

#[derive(Debug, Clone)]
pub struct Attribute{
    pub(crate) name: String,
    pub(crate) data: Vec<u8>
}

#[derive(Debug, Clone)]
pub enum Const{
    Str(String),
    Int(i32),
    Float(f32),
    Double(f64),
    Long(i64),
    Class(u16),
    FMIRef((u16, u16)),
    StrIndex(u16),
    NameAndType((u16, u16)),
    Invalid
}

#[derive(Debug, Clone)]
pub enum Types{
    Int(i32),
    Double(f64),
    Float(f32),
    Long(i64),
    Void,
    Class(String),
    Array((usize, ArrayTypes)),
    Boolean(bool),
    Str(String)
}

#[derive(Debug, Clone)]
pub struct ConstPool{
    pub consts: Vec<Const>
}

impl ConstPool{
    pub fn get(&self, idx:u16) -> Const{
        let idx = (idx-1) as usize;
        return self.consts[idx].clone();
    }
}

#[allow(non_snake_case)]
pub struct MethodAccessFlags{
    pub PUBLIC: bool,
    pub PRIVATE: bool,
    pub PROTECTED: bool,
    pub STATIC: bool,
    pub FINAL: bool,
    pub SYNCHRONIZED: bool,
    pub BRIDGE: bool,
    pub VARARGS: bool,
    pub NATIVE: bool,
    pub ABSTRACT: bool,
    pub STRICT: bool,
    pub SYNTHETIC: bool
}

impl MethodAccessFlags{
    pub fn new(flags: u16) -> Self{
        let f = Self{
            PUBLIC: flags & 0x1 != 0,
            PRIVATE: flags & 0x2 != 0,
            PROTECTED: flags & 0x4 != 0,
            STATIC: flags & 0x8 != 0,
            FINAL: flags & 0x10 != 0,
            SYNCHRONIZED: flags & 0x20 != 0,
            BRIDGE: flags & 0x40 != 0,
            VARARGS: flags & 0x80 != 0,
            NATIVE: flags & 0x100 != 0,
            ABSTRACT: flags & 0x400 != 0,
            STRICT: flags & 0x800 != 0,
            SYNTHETIC: flags & 0x1000 != 0
        };

        if f.PUBLIC && f.PRIVATE || f.PUBLIC && f.PROTECTED || f.PRIVATE && f.PROTECTED{
            panic!("Invalid flags.");
        }

        f
    }
}