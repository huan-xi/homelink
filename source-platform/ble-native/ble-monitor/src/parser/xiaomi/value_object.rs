use packed_struct::derive::PackedStruct;



use packed_struct::PackedStruct;



///lsbI16 类型的值
#[derive(PackedStruct, Debug, Clone)]
#[packed_struct(endian = "lsb")]
pub struct ValueLsbI16 {
    pub value: i16,
}
