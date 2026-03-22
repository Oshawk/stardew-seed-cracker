#[derive(Debug)]
pub struct ObjectData {
    pub name: &'static str,
    pub price: u32,
}

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));
