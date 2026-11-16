use std::path::{Path, PathBuf};

pub(crate) fn built_bee() -> PathBuf {
    find_bee_above(&std::env::current_exe().expect("test binary path"))
}

fn find_bee_above(start: &Path) -> PathBuf {
    let name = format!("bee{}", std::env::consts::EXE_SUFFIX);
    start
        .ancestors()
        .skip(1)
        .map(|dir| dir.join(&name))
        .find(|candidate| candidate.is_file())
        .unwrap_or_else(|| panic!("built bee binary not found above {}", start.display()))
}

#[cfg(test)]
mod tests {
    use super::find_bee_above;
    use std::path::{Path, PathBuf};

    fn bee_name() -> String {
        format!("bee{}", std::env::consts::EXE_SUFFIX)
    }

    fn profile_with_bee(root: &Path) -> PathBuf {
        let profile = root.join("target").join("release");
        std::fs::create_dir_all(&profile).unwrap();
        std::fs::write(profile.join(bee_name()), b"").unwrap();
        profile
    }

    #[test]
    fn finds_bee_from_the_deps_layout() {
        let tmp = tempfile::tempdir().unwrap();
        let profile = profile_with_bee(tmp.path());
        let deps = profile.join("deps");
        std::fs::create_dir_all(&deps).unwrap();
        assert_eq!(find_bee_above(&deps.join("x")), profile.join(bee_name()));
    }

    #[test]
    fn finds_bee_from_the_build_out_layout_past_a_bee_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let profile = profile_with_bee(tmp.path());
        let out = profile.join("build").join("bee").join("0123abcd").join("out");
        std::fs::create_dir_all(&out).unwrap();
        assert!(profile.join("build").join("bee").is_dir());
        assert_eq!(find_bee_above(&out.join("x")), profile.join(bee_name()));
    }

    #[test]
    fn panics_naming_the_start_when_no_bee_file_exists() {
        let tmp = tempfile::tempdir().unwrap();
        let start = tmp.path().join("target").join("release").join("deps").join("x");
        let err = std::panic::catch_unwind(|| find_bee_above(&start)).unwrap_err();
        let message = err.downcast_ref::<String>().cloned().unwrap_or_default();
        assert!(message.contains(&start.display().to_string()), "panic message: {message}");
    }
}
