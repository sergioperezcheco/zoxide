#![cfg(unix)]

use std::path::Path;

use assert_cmd::Command;
use tempfile::TempDir;

fn import(home: &Path, source: &str, vars: &[(&str, &str)], expected: &Path) {
    let data = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("zoxide").unwrap();
    cmd.env("HOME", home)
        .env("_ZO_DATA_DIR", data.path())
        .env("_ZO_EXCLUDE_DIRS", "")
        .env_remove("_FASD_DATA")
        .env_remove("_Z_DATA")
        .env_remove("ZSHZ_DATA");
    for (key, value) in vars {
        cmd.env(key, value);
    }
    cmd.args(["import", source]).assert().success();

    Command::cargo_bin("zoxide")
        .unwrap()
        .env("_ZO_DATA_DIR", data.path())
        .env("_ZO_EXCLUDE_DIRS", "")
        .args(["query", "--all", "--list"])
        .assert()
        .success()
        .stdout(format!("{}\n", expected.display()));
}

#[rstest::rstest]
#[case("fasd", "_FASD_DATA")]
#[case("z", "_Z_DATA")]
#[case("zsh-z", "ZSHZ_DATA")]
fn empty_import_paths_use_source_defaults(#[case] source: &str, #[case] variable: &str) {
    let home = TempDir::new().unwrap();
    let default = home.path().join("default-project");
    let custom = home.path().join("custom-project");
    let default_row = format!("{}|3|1700000000\n", default.display());
    std::fs::write(home.path().join(".fasd"), &default_row).unwrap();
    std::fs::write(home.path().join(".z"), &default_row).unwrap();
    let custom_file = home.path().join("custom-db");
    std::fs::write(&custom_file, format!("{}|2|1700000001\n", custom.display())).unwrap();
    let custom_file = custom_file.to_str().unwrap();

    import(home.path(), source, &[], &default);
    import(home.path(), source, &[(variable, "")], &default);
    import(home.path(), source, &[(variable, custom_file)], &custom);
    if source == "zsh-z" {
        import(home.path(), source, &[("ZSHZ_DATA", ""), ("_Z_DATA", custom_file)], &custom);
        import(home.path(), source, &[("ZSHZ_DATA", ""), ("_Z_DATA", "")], &default);
        import(home.path(), source, &[("_Z_DATA", custom_file)], &custom);
        import(home.path(), source, &[("ZSHZ_DATA", custom_file), ("_Z_DATA", "")], &custom);
        let default_file = home.path().join(".z");
        import(
            home.path(),
            source,
            &[("ZSHZ_DATA", custom_file), ("_Z_DATA", default_file.to_str().unwrap())],
            &custom,
        );
    }
}
