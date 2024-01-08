
use serde::Serialize;
use serde_json::ser::Formatter;
use std::io;
use std::io::Write;

struct I64ToStringFormatter;

impl Formatter for I64ToStringFormatter {
    fn write_i64<W>(&mut self, writer: &mut W, value: i64) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        write!(writer, "\"{}\"", value)
    }
}


pub fn to_json_string<T>(value: &T) -> io::Result<String>
where
    T: Serialize,
{
    let mut out = Vec::new();
    let formatter = I64ToStringFormatter;
    let mut ser = serde_json::Serializer::with_formatter(&mut out, formatter);
    value.serialize(&mut ser)?;
    Ok(String::from_utf8(out).unwrap())
}

#[derive(Serialize)]
struct S {
    x: i64,
}

#[test]
fn test() {
    let s = S { x: i64::MAX };
    let mut out = Vec::new();
    let formatter = I64ToStringFormatter;
    let mut ser = serde_json::Serializer::with_formatter(&mut out, formatter);
    s.serialize(&mut ser).unwrap();
    let a = std::str::from_utf8(&out).unwrap();
    println!("{}", a);
}
