use crate::diagnostics::Diagnostic;

use super::state::{
    OwnershipInfo,
    OwnershipState,
};

pub struct DropChecker;

impl DropChecker {
    pub fn drop_value(
        info: &mut OwnershipInfo,
        line: usize,
        column: usize,
    ) -> Result<(), Diagnostic> {
        match info.state {
            OwnershipState::Available => {
                info.state = OwnershipState::Dropped;
                Ok(())
            }

            OwnershipState::Moved => {
                Err(Diagnostic::error(
                    "AX028",
                    &format!(
                        "Cannot drop '{}': value has already been moved.",
                        info.name
                    ),
                    line,
                    column,
                ))
            }

            OwnershipState::Dropped => {
                Err(Diagnostic::error(
                    "AX029",
                    &format!(
                        "Cannot drop '{}': value has already been dropped.",
                        info.name
                    ),
                    line,
                    column,
                ))
            }

            _ => {
                Err(Diagnostic::error(
                    "AX030",
                    &format!(
                        "Cannot drop '{}': value is currently borrowed.",
                        info.name
                    ),
                    line,
                    column,
                ))
            }
        }
    }
}