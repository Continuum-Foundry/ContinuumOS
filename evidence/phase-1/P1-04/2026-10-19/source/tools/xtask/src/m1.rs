//! Minimal ISO integration; this is not a guest ELF loader or full M1 suite.
mod build;
mod cases;
mod inputs;
mod launch;
mod observe;
mod package;
mod process;
mod records;
mod support;

pub fn execute(action: &str) -> crate::Result<()> {
    cases::execute(action)
}
