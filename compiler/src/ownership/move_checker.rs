use crate::diagnostics::Diagnostic;

use super::state::{
    OwnershipInfo,
    OwnershipState,
};

pub struct MoveChecker;

impl MoveChecker {
    pub fn move_value(
        info: &mut OwnershipInfo,
        line: usize,
        column: usize,
    ) -> Result<(), Diagnostic> {
        if info.is_copy {
            return Ok(());
        }

        match info.state {
            OwnershipState::Available => {
                info.state = OwnershipState::Moved;
                Ok(())
            }

            OwnershipState::Moved => {
                Err(Diagnostic::error(
                    "AX020",
                    &format!(
                        "Cannot move '{}': value has already been moved.",
                        info.name
                    ),
                    line,
                    column,
                ))
            }

            _ => {
                Err(Diagnostic::error(
                    "AX021",
                    &format!(
                        "Cannot move '{}': value is not available.",
                        info.name
                    ),
                    line,
                    column,
                ))
            }
        }
    }

    pub fn check_usage(
        info: &OwnershipInfo,
        line: usize,
        column: usize,
    ) -> Result<(), Diagnostic> {
        match info.state {
            OwnershipState::Available => Ok(()),

            OwnershipState::Moved => {
                Err(Diagnostic::error(
                    "AX022",
                    &format!(
                        "Cannot use '{}': value has been moved.",
                        info.name
                    ),
                    line,
                    column,
                ))
            }

            OwnershipState::Dropped => {
                Err(Diagnostic::error(
                    "AX023",
                    &format!(
                        "Cannot use '{}': value has been dropped.",
                        info.name
                    ),
                    line,
                    column,
                ))
            }

            _ => {
                Err(Diagnostic::error(
                    "AX024",
                    &format!(
                        "Cannot access '{}': ownership is currently borrowed.",
                        info.name
                    ),
                    line,
                    column,
                ))
            }
        }
    }
}