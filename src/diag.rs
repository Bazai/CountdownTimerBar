//! The one place the app writes to stderr.

use std::fmt::Display;

/// Reports a problem the user cannot act on and the app can survive.
#[expect(
    clippy::print_stderr,
    reason = "menu-bar app: stderr is the only diagnostic sink"
)]
pub(crate) fn report(message: impl Display) {
    eprintln!("{message}");
}
