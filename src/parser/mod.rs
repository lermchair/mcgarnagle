mod bristol;
mod yosys;
use crate::utils::GateType;

pub use self::bristol::*;
pub use self::yosys::*;

#[derive(Debug)]
pub struct Gate {
    inputs: Vec<String>,
    output: String,
    type_: GateType,
}
