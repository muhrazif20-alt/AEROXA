use crate::diagnostics::Diagnostic;

use super::state::{
    OwnershipInfo,
    OwnershipState,
};

pub struct BorrowChecker;

impl BorrowChecker {
    pub fn borrow(
        info: &mut OwnershipInfo,
        line: usize,
        column: usize,
    ) -> Result<(), Diagnostic> {
        match info.state {
            OwnershipState::Available => {
                info.state = OwnershipState::Borrowed;
                Ok(())
            }

            OwnershipState::Borrowed => {
                Ok(())
            }

            _ => {
                Err(Diagnostic::error(
                    "AX025",
                    &format!(
                        "Cannot borrow '{}': value is not available.",
                        info.name
                    ),
                    line,
                    column,
                ))
            }
        }
    }

    pub fn borrow_mut(
        info: &mut OwnershipInfo,
        line: usize,
        column: usize,
    ) -> Result<(), Diagnostic> {
        match info.state {
            OwnershipState::Available => {
                info.state = OwnershipState::MutablyBorrowed;
                Ok(())
            }

            _ => {
                Err(Diagnostic::error(
                    "AX026",
                    &format!(
                        "Cannot mutably borrow '{}': value is already borrowed or unavailable.",
                        info.name
                    ),
                    line,
                    column,
                ))
            }
        }
    }

    pub fn release(
        info: &mut OwnershipInfo,
    ) {
        match info.state {
            OwnershipState::Borrowed
            | OwnershipState::MutablyBorrowed => {
                info.state = OwnershipState::Available;
            }

            _ => {}
        }
    }
}