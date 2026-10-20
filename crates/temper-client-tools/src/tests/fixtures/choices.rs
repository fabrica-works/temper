use super::*;
/// The chosen main differs from both the template main and its sub-agent.
pub(crate) const MODEL_NAME: &[u8] = b"chosen-model";
pub(crate) const EFFORTLESS_MODEL_NAME: &[u8] = b"plain-model";
pub(crate) const EFFORT_NAME: &[u8] = b"high";
pub(crate) const MODEL_CHAT: &[&str] =
    &["chat", "hello", "--project", "shop", "--model", "chosen-model", "--effort", "high", "--key", "model-chat"];
pub(crate) const EFFORTLESS_CHAT: &[&str] =
    &["chat", "hello", "--project", "shop", "--model", "plain-model", "--key", "plain-chat"];
pub(crate) fn model_effort(model: &[u8], effort: Option<&[u8]>) -> c::ModelEffort {
    let model = act_text(model);
    let Some(effort) = effort else { return c::ModelEffort { model, effort: None } };
    c::ModelEffort { model, effort: Some(act_text(effort)) }
}
pub(crate) fn prior_choice() -> c::LlmChoice {
    let mut subagents = List::with_capacity(1);
    subagents.push(model_effort(b"small-model", Some(b"low"))).expect("one sub-agent choice");
    c::LlmChoice { account: Some(43), main: model_effort(b"template-model", Some(b"low")), subagents }
}
pub(crate) fn choice_me(budget: u64) -> c::Me {
    let mut me = chat_me(budget);
    let project = me.projects.get_mut(0).expect("project");
    project.templates.get_mut(0).expect("chat template").llm = Some(prior_choice());
    project.models = List::with_capacity(2);
    let mut efforts = List::with_capacity(1);
    efforts.push(act_text(EFFORT_NAME)).expect("one effort");
    project
        .models
        .push(c::MeModel { name: act_text(MODEL_NAME), provider: None, window: 8192, output: 1024, efforts })
        .expect("one selectable model");
    project
        .models
        .push(c::MeModel {
            name: act_text(EFFORTLESS_MODEL_NAME),
            provider: None,
            window: 8192,
            output: 1024,
            efforts: List::with_capacity(0),
        })
        .expect("effortless model");
    me
}
pub(crate) fn choice_root(me: c::Me) -> Harness {
    let mut root = started(List::with_capacity(EMPTY));
    root.deliver(c::Delivery::Me(Box::new(me)));
    root
}
pub(crate) fn chat_task(root: &Harness, call: u32) -> &c::NewTask {
    let c::Ask::Create { tasks, .. } = act_ask(root, call) else { panic!("chat creates") };
    tasks.get(0).expect("one chat task")
}
