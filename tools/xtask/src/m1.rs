//! Offline ISO integration and the bounded M1 acceptance matrix.
mod build;
mod cases;
mod inputs;
mod launch;
mod observe;
mod package;
mod process;
mod records;
mod suite;
mod support;

pub fn execute(action: &str) -> crate::Result<()> {
    cases::execute(action)
}
