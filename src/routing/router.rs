use crate::action::Action;
use crate::error::HttpError;
use crate::routing::route::{Reconciled, Route};
use hyper::body::Incoming;
use hyper::{Method, Request};
use std::cell::RefCell;

#[derive(Debug)]
pub struct Router {
    pub routes: Vec<Route>,
}

impl Router {
    pub fn new(register_routes: fn(&mut Router)) -> Router {
        let mut router = Router { routes: Vec::new() };

        register_routes(&mut router);

        router
    }

    pub fn add<A: Action + 'static>(
        &mut self,
        method: Method,
        path: &str,
        action: A,
        modifier: Option<fn(Route) -> Route>,
    ) -> &mut Router {
        let mut route = Route::new(method, path.to_owned(), action);

        if let Some(modifier) = modifier {
            route = modifier(route);
        }

        self.routes.push(route);

        self
    }

    pub fn get<A: Action + 'static>(&mut self, path: &str, action: A) -> &mut Router {
        self.add(Method::GET, path, action, None);

        self
    }

    pub fn getm<A: Action + 'static>(
        &mut self,
        path: &str,
        action: A,
        modifier: fn(Route) -> Route,
    ) -> &mut Router {
        self.add(Method::GET, path, action, Some(modifier));

        self
    }

    pub fn getn<A: Action + 'static>(
        &mut self,
        path: &str,
        action: A,
        name: String,
    ) -> &mut Router {
        self.add(Method::GET, path, action, Some(|route| route.name(name)));

        self
    }

    pub fn post<A: Action + 'static>(
        &mut self,
        path: &str,
        action: A,
        modifier: Option<fn(Route) -> Route>,
    ) -> &mut Router {
        self.add(Method::POST, path, action, modifier);

        self
    }

    pub fn postm<A: Action + 'static>(
        &mut self,
        path: &str,
        action: A,
        modifier: fn(Route) -> Route,
    ) -> &mut Router {
        self.add(Method::POST, path, action, Some(modifier));

        self
    }

    pub fn postn<A: Action + 'static>(
        &mut self,
        path: &str,
        action: A,
        name: String,
    ) -> &mut Router {
        self.add(Method::POST, path, action, Some(|route| route.name(name)));

        self
    }

    pub fn patch<A: Action + 'static>(
        &mut self,
        path: &str,
        action: A,
        modifier: Option<fn(Route) -> Route>,
    ) -> &mut Router {
        self.add(Method::PATCH, path, action, modifier);

        self
    }

    pub fn patchm<A: Action + 'static>(
        &mut self,
        path: &str,
        action: A,
        modifier: fn(Route) -> Route,
    ) -> &mut Router {
        self.add(Method::PATCH, path, action, Some(modifier));

        self
    }

    pub fn patchn<A: Action + 'static>(
        &mut self,
        path: &str,
        action: A,
        name: String,
    ) -> &mut Router {
        self.add(Method::PATCH, path, action, Some(|route| route.name(name)));

        self
    }

    pub fn put<A: Action + 'static>(
        &mut self,
        path: &str,
        action: A,
        modifier: Option<fn(Route) -> Route>,
    ) -> &mut Router {
        self.add(Method::PUT, path, action, modifier);

        self
    }

    pub fn putm<A: Action + 'static>(
        &mut self,
        path: &str,
        action: A,
        modifier: fn(Route) -> Route,
    ) -> &mut Router {
        self.add(Method::PUT, path, action, Some(modifier));

        self
    }

    pub fn putn<A: Action + 'static>(
        &mut self,
        path: &str,
        action: A,
        name: String,
    ) -> &mut Router {
        self.add(Method::PUT, path, action, Some(|route| route.name(name)));

        self
    }

    pub fn delete<A: Action + 'static>(
        &mut self,
        path: &str,
        action: A,
        modifier: Option<fn(Route) -> Route>,
    ) -> &mut Router {
        self.add(Method::DELETE, path, action, modifier);

        self
    }

    pub fn deletem<A: Action + 'static>(
        &mut self,
        path: &str,
        action: A,
        modifier: fn(Route) -> Route,
    ) -> &mut Router {
        self.add(Method::DELETE, path, action, Some(modifier));

        self
    }

    pub fn deleten<A: Action + 'static>(
        &mut self,
        path: &str,
        action: A,
        name: String,
    ) -> &mut Router {
        self.add(Method::DELETE, path, action, Some(|route| route.name(name)));

        self
    }

    pub fn head<A: Action + 'static>(
        &mut self,
        path: &str,
        action: A,
        modifier: Option<fn(Route) -> Route>,
    ) -> &mut Router {
        self.add(Method::HEAD, path, action, modifier);

        self
    }

    pub fn headm<A: Action + 'static>(
        &mut self,
        path: &str,
        action: A,
        modifier: fn(Route) -> Route,
    ) -> &mut Router {
        self.add(Method::HEAD, path, action, Some(modifier));

        self
    }

    pub fn headn<A: Action + 'static>(
        &mut self,
        path: &str,
        action: A,
        name: String,
    ) -> &mut Router {
        self.add(Method::HEAD, path, action, Some(|route| route.name(name)));

        self
    }

    pub fn connect<A: Action + 'static>(
        &mut self,
        path: &str,
        action: A,
        modifier: Option<fn(Route) -> Route>,
    ) -> &mut Router {
        self.add(Method::CONNECT, path, action, modifier);

        self
    }

    pub fn connectm<A: Action + 'static>(
        &mut self,
        path: &str,
        action: A,
        modifier: fn(Route) -> Route,
    ) -> &mut Router {
        self.add(Method::CONNECT, path, action, Some(modifier));

        self
    }

    pub fn connectn<A: Action + 'static>(
        &mut self,
        path: &str,
        action: A,
        name: String,
    ) -> &mut Router {
        self.add(Method::CONNECT, path, action, Some(|route| route.name(name)));

        self
    }

    pub fn options<A: Action + 'static>(
        &mut self,
        path: &str,
        action: A,
        modifier: Option<fn(Route) -> Route>,
    ) -> &mut Router {
        self.add(Method::OPTIONS, path, action, modifier);

        self
    }

    pub fn optionsm<A: Action + 'static>(
        &mut self,
        path: &str,
        action: A,
        modifier: fn(Route) -> Route,
    ) -> &mut Router {
        self.add(Method::OPTIONS, path, action, Some(modifier));

        self
    }

    pub fn optionsn<A: Action + 'static>(
        &mut self,
        path: &str,
        action: A,
        name: String,
    ) -> &mut Router {
        self.add(Method::OPTIONS, path, action, Some(|route| route.name(name)));

        self
    }

    pub fn trace<A: Action + 'static>(
        &mut self,
        path: &str,
        action: A,
        modifier: Option<fn(Route) -> Route>,
    ) -> &mut Router {
        self.add(Method::TRACE, path, action, modifier);

        self
    }

    pub fn tracem<A: Action + 'static>(
        &mut self,
        path: &str,
        action: A,
        modifier: fn(Route) -> Route,
    ) -> &mut Router {
        self.add(Method::TRACE, path, action, Some(modifier));

        self
    }

    pub fn tracen<A: Action + 'static>(
        &mut self,
        path: &str,
        action: A,
        name: String,
    ) -> &mut Router {
        self.add(Method::TRACE, path, action, Some(|route| route.name(name)));

        self
    }

    pub fn resolve(
        &self,
        request: &Request<Incoming>,
    ) -> Result<Option<(&Route, RefCell<Vec<Reconciled>>)>, HttpError> {
        self.resolve_inner(request.uri().path(), request.method())
    }

    /// This lets us test without requiring a Request<Incoming> instance
    pub fn resolve_inner(
        &self,
        path: &str,
        method: &Method,
    ) -> Result<Option<(&Route, RefCell<Vec<Reconciled>>)>, HttpError> {
        // todo need to get all routes that match, then we need to check to see if
        // todo request method matches any of them. e.g. PUT request must have a
        // todo route with a PUT.

        // todo first filter by requested method. if none found, see if there is another route
        // todo with the same path. if match found, we know to return a 405 method not allowed error.
        for route in &self.routes {
            let (is_match, reconciled) = route.is_match(path);
            if is_match {
                return if method != route.get_method() {
                    Err(HttpError::new(405, "Method not allowed".to_string()))
                } else {
                    Ok(Some((route, reconciled)))
                };
            }
        }

        Ok(None)
    }
}
