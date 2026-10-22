//! Executable GitHub git path: domain/testing.md, 2.6.
pub mod support;

#[test]
fn github_clone_and_push_cross_the_engine_worker_and_git_processes() {
    support::github::push();
}
