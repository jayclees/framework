use crate::env::{AppEnv, Env};
use crate::http::error::HttpError;
use crate::http::request::HttpRequest;
use crate::routing::router::Router;
use crate::support::logger::Logger;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response};
use minijinja::context;
use minijinja_autoreload::AutoReloader;
use sea_orm::DatabaseConnection;
use serde::Serialize;
use serde_json::json;
use std::any::Any;
use std::sync::Arc;
use tokio::net::TcpListener;

pub(crate) type AppState = Box<dyn Any + Send + Sync>;

pub struct App {
    env: Env,
    router: Arc<Router>,
    listener: TcpListener,
    template: Option<AutoReloader>,
    db: Option<DatabaseConnection>,
    logger: Logger,
    state: AppState,
}

impl App {
    pub fn init(
        env: Env,
        router: Arc<Router>,
        listener: TcpListener,
        template: Option<AutoReloader>,
        db: Option<DatabaseConnection>,
        logger: Logger,
        state: AppState,
    ) -> App {
        App {
            env,
            router,
            listener,
            template,
            db,
            logger,
            state,
        }
    }

    pub fn listener(&self) -> &TcpListener {
        &self.listener
    }

    pub fn state<A: Any>(&self) -> &A {
        &self.state.downcast_ref::<A>().unwrap()
    }

    pub fn template<S: Serialize>(&self, name: &str, context: S) -> Result<String, minijinja::Error>
    where
        minijinja::Value: From<S>,
    {
        let value = context! {
            env => self.env,
            ..context
        };
        self.template
            .as_ref()
            .expect("Template reloader not set.")
            .acquire_env()
            .expect("Failed to resolve minijinja environment")
            .get_template(name)?
            .render(value)
    }

    pub fn db(&self) -> Option<&DatabaseConnection> {
        self.db.as_ref()
    }

    pub async fn dispatch(
        &self,
        request: Request<Incoming>,
    ) -> Option<Result<Response<Full<Bytes>>, HttpError>> {
        let result = &self.router.resolve(&request);
        match result {
            Ok(route) => match route {
                Some((route, reconciled)) => {
                    // todo this is the spot to implement the bus pattern
                    match route
                        .action()
                        .handle(&self, HttpRequest::new(request, reconciled.take()))
                        .await
                    {
                        Ok(result) => {
                            route.action().log().await;
                            Some(result.to_response())
                        }
                        Err(e) => Some(Err(e)),
                    }
                }
                None => Some(Err(HttpError::new(404, "Page not found".to_string()))),
            },
            Err(error) => {
                // todo figure out why the error param is passed by reference
                Some(Err(error.clone()))
            }
        }
    }

    pub fn is_production(&self) -> bool {
        self.env.env() == &AppEnv::Production
    }

    pub fn is_local(&self) -> bool {
        self.env.env() == &AppEnv::Local
    }

    pub(crate) fn error(&self, error: &HttpError, json: bool) -> Result<Response<Full<Bytes>>, HttpError> {
        let mut builder = Response::builder().status(error.code());
        let msg = if self.is_local() {
            error.message()
        } else {
            self.error_message(error.code())
        };
        let content = if json {
            builder = builder.header("Content-Type", "application/json");
            json!({"code": error.code(), "message": msg}).to_string()
        } else {
            self.template(
                "errors/default.html",
                context!(code => error.code(), message => msg),
            )
            .unwrap()
        };

        Ok(builder.body(Full::new(Bytes::from(content))).unwrap())
    }

    fn error_message(&self, code: u16) -> String {
        let string = format!("Unknown error ({}).", code);
        match code {
            400 => "Bad request",
            401 => "Forbidden",
            402 => "Payment required",
            403 => "Unauthorized",
            404 => "Not found",
            405 => "Method not allowed",
            409 => "Conflict",
            410 => "Gone",
            411 => "Length required",
            412 => "Precondition failed",
            413 => "Content too large",
            414 => "URI too long",
            415 => "Unsupported media type",
            416 => "Range not satisfiable",
            417 => "Expectation failed",
            418 => "I'm a teapot",
            421 => "Misdirected request",
            422 => "Unprocessable content",
            423 => "Locked",
            424 => "Failed dependency",
            425 => "Too early",
            426 => "Upgrade required",
            428 => "Precondition required",
            429 => "Too many requests",
            431 => "Request header fields too large",
            451 => "Unavailable for legal reasons",
            400..=499 => "Client error",
            500 => "Server error",
            501 => "Not implemented",
            502 => "Bad gateway",
            503 => "Service unavailable",
            504 => "Gateway timeout",
            505 => "HTTP version not supported",
            506 => "Variant also negotiates",
            507 => "Insufficient storage",
            508 => "Loop detected",
            510 => "Not extended",
            511 => "Network authentication required",
            500..=599 => "Something went wrong",
            _ => string.as_str(),
        }
        .to_owned()
    }
}
