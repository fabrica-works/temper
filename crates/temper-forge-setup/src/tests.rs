mod fetch;
mod files;
mod fixtures;
mod given;
mod golden;
mod limits;
mod members;
mod prepare;
mod read;
mod run;
mod spec;
mod steps;
mod upgrade;

#[test]
fn starter_workflow_is_pinned_to_the_setup_design() {
    let design = include_str!("../../../docs/design/protocol/setup.md");
    let first = design.split("### 6.7 The first repository").nth(1).unwrap();
    let block = first.split("  ```").nth(1).unwrap().strip_prefix('\n').unwrap();
    let mut bytes = String::new();
    for line in block.lines() {
        bytes.push_str(line.strip_prefix("  ").unwrap());
        bytes.push('\n');
    }
    assert_eq!(crate::STARTER_WORKFLOW, bytes.as_bytes());
    assert_eq!(crate::STARTER_PATH, b".forgejo/workflows/ci.yml");
}
