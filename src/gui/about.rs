//! What the About card shows: the version and the copyright notice.

use gpui_kit::{App, Global, SharedString};

/// Replaces the real values, so snapshots do not change with every release or new year.
pub(crate) struct AboutOverride {
    pub version: SharedString,
    pub copyright: SharedString,
}

impl Global for AboutOverride {}

pub(crate) fn version(cx: &App) -> SharedString {
    cx.try_global::<AboutOverride>()
        .map_or_else(|| crate::VERSION.into(), |about| about.version.clone())
}

pub(crate) fn copyright(cx: &App) -> SharedString {
    cx.try_global::<AboutOverride>().map_or_else(
        || crate::copyright().into(),
        |about| about.copyright.clone(),
    )
}
