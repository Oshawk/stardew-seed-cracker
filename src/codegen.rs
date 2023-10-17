#[derive(Debug)]
pub struct ObjectInformation {
    pub name: &'static str,
    pub price: u16,
    pub edibility: i16,
    pub type_and_category: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));
