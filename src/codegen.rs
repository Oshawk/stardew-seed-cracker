#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectClass {
    /// Passes both Layer 1 and Layer 2 filters.
    /// The u16 is the index into ELIGIBLE_OBJECTS (0..333).
    FullyEligible(u16),
    /// Passes Layer 1 but fails Layer 2.
    Intermediate,
    /// Fails Layer 1.
    Ineligible,
}

#[derive(Debug, Clone, Copy)]
pub struct EnumerationEntry {
    pub class: ObjectClass,
}

#[derive(Debug, Clone, Copy)]
pub struct EligibleObject {
    pub id: u16,
    pub name: &'static str,
    pub price: u16,
}

#[allow(dead_code)]
mod generated {
    use super::*;
    include!(concat!(env!("OUT_DIR"), "/codegen.rs"));
}
pub use generated::*;
