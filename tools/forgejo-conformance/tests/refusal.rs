#[test]
fn an_unset_forgejo_variable_is_refused_naming_it() {
    let tmp = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"));
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_forgejo-conformance"))
        .args(["--scratch"])
        .arg(tmp.join("refusal-scratch"))
        .arg("--out")
        .arg(tmp.join("refusal-out"))
        .env_remove("TEMPER_FORGEJO")
        .output()
        .expect("the tool runs");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stderr, b"Error: \"TEMPER_FORGEJO must name the pinned Forgejo\"\n");
}
