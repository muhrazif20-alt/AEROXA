#[derive(Debug, Clone, PartialEq)]
pub enum OwnershipState {
    Available,
    Moved,
    Borrowed,
    MutablyBorrowed,
    Dropped,
}

#[derive(Debug, Clone)]
pub struct OwnershipInfo {
    pub name: String,
    pub data_type: String,
    pub state: OwnershipState,
    pub is_copy: bool,
}

impl OwnershipInfo {
    pub fn new(
        name: String,
        data_type: String,
    ) -> Self {
        let is_copy = matches!(
            data_type.as_str(),
            "Int8"
                | "Int16"
                | "Int32"
                | "Int64"
                | "Float32"
                | "Float64"
                | "Bool"
        );

        Self {
            name,
            data_type,
            state: OwnershipState::Available,
            is_copy,
        }
    }
}