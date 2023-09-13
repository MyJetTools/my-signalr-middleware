use my_json::json_writer::JsonObjectWriter;

pub enum SignalRParam<'s> {
    JsonObject(&'s JsonObjectWriter),
    String(&'s str),
    Number(i64),
    Float(f64),
    Boolean(bool),
    Raw(&'s [Vec<u8>]),
    None,
}
