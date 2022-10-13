use num_derive::FromPrimitive;

#[derive(FromPrimitive)]
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
pub struct Const{
    pub tag: u8,
    pub data: ConstTypes
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
pub enum ConstTypes{
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
    Array((Vec<Types>, ArrayTypes)),
    Boolean(bool)
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