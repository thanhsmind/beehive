use std::path::{Path, PathBuf};

pub(crate) fn built_bee() -> PathBuf {
    let start = std::env::current_exe().expect("test binary path");
    find_bee_above(&start).unwrap_or_else(|| panic!("built bee binary not found above {}", start.display()))
}

fn find_bee_above(start: &Path) -> Option<PathBuf> {
    let name = format!("bee{}", std::env::consts::EXE_SUFFIX);
    start.ancestors().skip(1).map(|dir| dir.join(&name)).find(|candidate| candidate.is_file())
}

#[cfg(test)]
mod tests {
    use super::find_bee_above;

    #[test]
    fn finds_the_nearest_ancestor_holding_a_bee_file() {
        let tmp = tempfile::tempdir().unwrap();
        let profile = tmp.path().join("release");
        let nested = profile.join("build").join("bee").join("hash").join("out");
        std::fs::create_dir_all(&nested).unwrap();
        let name = format!("bee{}", std::env::consts::EXE_SUFFIX);
        std::fs::write(profile.join(&name), b"").unwrap();
        let start = nested.join("bee-test");
        assert_eq!(find_bee_above(&start), Some(profile.join(&name)));
    }

    #[test]
    fn returns_none_when_no_ancestor_holds_one() {
        let tmp = tempfile::tempdir().unwrap();
        let start = tmp.path().join("a").join("b").join("bee-test");
        assert_eq!(find_bee_above(&start), None);
    }
}
