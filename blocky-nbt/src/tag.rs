#[cfg(feature = "preserve_order")]
pub type Map<T> = indexmap::IndexMap<String, T>;
#[cfg(not(feature = "preserve_order"))]
pub type Map<T> = std::collections::HashMap<String, T>;

#[derive(Debug, Clone, PartialEq)]
pub enum NbtTag {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List(NbtList),
    Compound(NbtCompound),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl NbtTag {
    pub fn id(&self) -> u8 {
        match self {
            Self::Byte(_) => 0x01,
            Self::Short(_) => 0x02,
            Self::Int(_) => 0x03,
            Self::Long(_) => 0x04,
            Self::Float(_) => 0x05,
            Self::Double(_) => 0x06,
            Self::ByteArray(_) => 0x07,
            Self::String(_) => 0x08,
            Self::List(_) => 0x09,
            Self::Compound(_) => 0x0A,
            Self::IntArray(_) => 0x0B,
            Self::LongArray(_) => 0x0C,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Byte(_) => "BYTE",
            Self::Short(_) => "SHORT",
            Self::Int(_) => "INT",
            Self::Long(_) => "LONG",
            Self::Float(_) => "FLOAT",
            Self::Double(_) => "DOUBLE",
            Self::ByteArray(_) => "BYTE[]",
            Self::String(_) => "STRING",
            Self::List(_) => "LIST",
            Self::Compound(_) => "COMPOUND",
            Self::IntArray(_) => "INT[]",
            Self::LongArray(_) => "LONG[]",
        }
    }

    pub fn pretty_name(&self) -> &str {
        match self {
            Self::Byte(_) => "TAG_Byte",
            Self::Short(_) => "TAG_Short",
            Self::Int(_) => "TAG_Int",
            Self::Long(_) => "TAG_Long",
            Self::Float(_) => "TAG_Float",
            Self::Double(_) => "TAG_Double",
            Self::ByteArray(_) => "TAG_Byte_Array",
            Self::String(_) => "TAG_String",
            Self::List(_) => "TAG_List",
            Self::Compound(_) => "TAG_Compound",
            Self::IntArray(_) => "TAG_Int_Array",
            Self::LongArray(_) => "TAG_Long_Array",
        }
    }

    pub fn byte_len(&self) -> usize {
        match self {
            Self::Byte(_) => 1,
            Self::Short(_) => 2,
            Self::Int(_) => 4,
            Self::Long(_) => 8,
            Self::Float(_) => 4,
            Self::Double(_) => 8,
            Self::ByteArray(v) => v.len(),
            Self::String(s) => 2 + s.len(),
            Self::List(l) => todo!(),
            Self::Compound(c) => todo!(),
            Self::IntArray(a) => 4 + 4 * a.len(),
            Self::LongArray(a) => 4 + 8 * a.len(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NbtList(Vec<NbtTag>);

#[derive(Debug, Clone, PartialEq)]
pub struct NbtCompound(Map<NbtTag>);

macro_rules! impl_from {
    ($typ:ty, $tag:ident) => {
        impl From<$typ> for NbtTag {
            fn from(value: $typ) -> Self {
                Self::$tag(value)
            }
        }
    };
}

impl_from!(i8, Byte);
impl_from!(i16, Short);
impl_from!(i32, Int);
impl_from!(i64, Long);
impl_from!(f32, Float);
impl_from!(f64, Double);
impl_from!(Vec<i8>, ByteArray);
impl_from!(String, String);
impl_from!(NbtList, List);
impl_from!(NbtCompound, Compound);
impl_from!(Vec<i32>, IntArray);
impl_from!(Vec<i64>, LongArray);
