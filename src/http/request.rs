use crate::routing::route::Reconciled;
use hyper::body::Incoming;
use hyper::Request;
use std::collections::HashMap;

pub struct HttpRequest {
    inner: Request<Incoming>,
    variables: HashMap<String, String>,
}

impl HttpRequest {
    pub fn new(request: Request<Incoming>, reconciled: Vec<Reconciled>) -> HttpRequest {
        let mut hash_map = HashMap::new();
        for item in reconciled {
            match item {
                Reconciled::Variable { handle, value } => {
                    hash_map.insert(handle, value);
                }
                _ => {}
            }
        }
        HttpRequest {
            inner: request,
            variables: hash_map,
        }
    }

    pub fn inner(&self) -> &Request<Incoming> {
        &self.inner
    }

    pub fn var(&self, handle: &str) -> Option<&String> {
        self.variables.get(handle)
    }
}
