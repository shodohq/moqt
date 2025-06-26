#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SetupParameter {
    pub parameter_type: u64,
    pub value: Vec<u8>,
}

