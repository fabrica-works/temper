//! A story owns its whole documents and the locations of their deferred addresses.
use super::{documents, files, key, private};
use crate::{cluster::Shape, forge, github};
use serde_json::Value;
use std::{net::SocketAddr, path::Path};

pub(crate) struct Documents {
    agent: Value,
    standalone: BoundDocument,
    cluster: Option<BoundDocument>,
}
impl Documents {
    pub(crate) fn new(
        path: &Path,
        address: SocketAddr,
        direct_key: bool,
        forge: Option<forge::Fixture>,
        github: Option<github::Fixture>,
        clients: Option<SocketAddr>,
        shape: Option<&Shape>,
    ) -> Self {
        files(path).expect("stand-in files");
        // A distinct construction address identifies only forge-derived strings,
        // including customized grants. Listener sockets and the LLM stay distinct.
        // No peer is started or contacted at this address.
        let marker = SocketAddr::from(([127, 0, 0, 2], 0));
        let (mut agent, mut standalone) = documents(path, address, marker);
        if direct_key {
            key(&mut agent, &mut standalone);
        }
        if let Some(fixture) = forge {
            forge::configure(&mut standalone, &mut agent, marker, &fixture);
            forge::secrets(path).expect("forge secrets");
        }
        if let Some(fixture) = github {
            github::configure(&mut standalone, &mut agent, marker, clients.expect("fixed clients"), fixture);
        }
        let cluster = shape.map(|shape| {
            BoundDocument::new(
                shape.documents(path.parent().expect("cluster scratch"), standalone.clone(), &mut agent),
                marker,
            )
        });
        let mut built = Self { agent, standalone: BoundDocument::new(standalone, marker), cluster };
        built.bind(address, address, shape);
        built
    }
    pub(crate) fn bind(&mut self, llm: SocketAddr, forge: SocketAddr, shape: Option<&Shape>) {
        self.agent["endpoints"][0]["address"] = llm.to_string().into();
        for document in std::iter::once(&mut self.standalone).chain(self.cluster.iter_mut()) {
            document.bind(forge);
            if !document.value["engine"].is_null() {
                document.value["engine"]["llm"]["endpoints"][0]["address"]["socket"] = llm.to_string().into();
            }
        }
        if let Some(Shape::Worker { engine, .. }) = shape {
            self.cluster.as_mut().expect("worker document").value["workers"][0]["engine"]["socket"] =
                engine.unwrap_or(([127, 0, 0, 1], 0).into()).to_string().into();
        }
    }
    pub(crate) fn write(&self, path: &Path) {
        for (name, value) in [("agent.json", &self.agent), ("temper.json", &self.standalone.value)] {
            private(&path.join(name), &serde_json::to_vec(value).expect("fixture JSON"))
                .expect("private fixture document");
        }
        if let Some(document) = &self.cluster {
            private(
                &path.parent().expect("cluster scratch").join("config/temper.json"),
                &serde_json::to_vec(&document.value).expect("cluster JSON"),
            )
            .expect("private cluster document");
        }
    }
}

struct BoundDocument {
    value: Value,
    sockets: Vec<String>,
    origins: Vec<String>,
}
impl BoundDocument {
    fn new(value: Value, marker: SocketAddr) -> Self {
        let mut built = Self { value, sockets: Vec::new(), origins: Vec::new() };
        bindings(
            &built.value,
            "",
            &marker.to_string(),
            &format!("http://{marker}"),
            &mut built.sockets,
            &mut built.origins,
        );
        built
    }
    fn bind(&mut self, address: SocketAddr) {
        for (paths, value) in [(&self.sockets, address.to_string()), (&self.origins, format!("http://{address}"))] {
            for path in paths {
                *self.value.pointer_mut(path).expect("built address location") = value.clone().into();
            }
        }
    }
}

fn bindings(
    value: &Value,
    path: &str,
    socket: &str,
    origin: &str,
    sockets: &mut Vec<String>,
    origins: &mut Vec<String>,
) {
    match value {
        Value::String(value) if value == socket => sockets.push(path.into()),
        Value::String(value) if value == origin => origins.push(path.into()),
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                bindings(value, &format!("{path}/{index}"), socket, origin, sockets, origins);
            }
        }
        Value::Object(values) => {
            for (name, value) in values {
                let name = name.replace('~', "~0").replace('/', "~1");
                bindings(value, &format!("{path}/{name}"), socket, origin, sockets, origins);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::BoundDocument;
    use serde_json::json;

    #[test]
    fn binding_keeps_listeners_and_other_stories_separate_even_after_addresses_coincide() {
        let marker = ([127, 0, 0, 2], 0).into();
        let initial = ([127, 0, 0, 1], 0).into();
        let value = json!({"listener":"127.0.0.1:0", "a~/b":["127.0.0.2:0", "http://127.0.0.2:0"]});
        let mut first = BoundDocument::new(value.clone(), marker);
        let mut second = BoundDocument::new(value, marker);
        first.bind(initial);
        second.bind(initial);
        first.bind(([127, 0, 0, 1], 12345).into());
        let first_bytes = serde_json::to_vec(&first.value).expect("first JSON");
        second.bind(([127, 0, 0, 1], 54321).into());
        assert_eq!(first_bytes, serde_json::to_vec(&first.value).expect("unchanged first JSON"));
        for (document, port) in [(first, 12345), (second, 54321)] {
            assert_eq!(
                document.value,
                json!({
                    "listener":"127.0.0.1:0", "a~/b":[format!("127.0.0.1:{port}"), format!("http://127.0.0.1:{port}")]
                })
            );
        }
    }
}
