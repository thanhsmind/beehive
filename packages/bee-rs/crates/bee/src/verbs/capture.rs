// bee capture — STUB: every argv shape falls through to the Node delegate.

use std::ffi::OsString;
use std::process::ExitCode;
use std::time::Instant;

pub fn try_native(_args: &[OsString], _t0: Instant) -> Option<ExitCode> {
    None
}
