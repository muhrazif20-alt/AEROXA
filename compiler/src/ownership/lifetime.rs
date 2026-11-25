use crate::diagnostics::Diagnostic;

#[derive(Debug, Clone, PartialEq)]
pub struct LifetimeInfo {
    pub owner: String,
    pub reference: String,
    pub owner_scope: usize,
    pub reference_scope: usize,
}

pub struct LifetimeChecker;

impl LifetimeChecker {
    pub fn check(
        lifetime: &LifetimeInfo,
        line: usize,
        column: usize,
    ) -> Result<(), Diagnostic> {
        if lifetime.reference_scope > lifetime.owner_scope {
            return Err(Diagnostic::error(
                "AX027",
                &format!(
                    "Reference '{}' cannot outlive its owner '{}'.",
                    lifetime.reference,
                    lifetime.owner
                ),
                line,
                column,
            ));
        }

        Ok(())
    }
}