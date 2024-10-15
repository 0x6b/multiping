use std::{collections::BTreeMap, sync::LazyLock};

use indicatif::ProgressStyle;

const TICK_CHARS: &str = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏";

pub struct ProgressStyleMap {
    inner: BTreeMap<&'static str, ProgressStyle>,
}

impl ProgressStyleMap {
    pub fn get(&self, key: &str) -> ProgressStyle {
        self.inner.get(key).unwrap().clone()
    }
}

pub static PROGRESS_STYLE_MAP: LazyLock<ProgressStyleMap> = LazyLock::new(|| {
    let mut inner = BTreeMap::new();
    inner.insert(
        "default",
        ProgressStyle::with_template("{spinner:.bold} {prefix:.green}: {wide_msg}")
            .unwrap()
            .tick_chars(TICK_CHARS),
    );
    inner.insert(
        "error",
        ProgressStyle::with_template("{spinner:.bold} {prefix:.red}: {wide_msg}")
            .unwrap()
            .tick_chars(TICK_CHARS),
    );
    ProgressStyleMap { inner }
});
