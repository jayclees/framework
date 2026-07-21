use crate::http::error::HttpError;
use crate::http::request::HttpRequest;
use crate::routing::router::Router;
use crate::support::logger::Logger;
use futures::FutureExt;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::rt::Executor;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto;
use minijinja::context;
use minijinja_autoreload::AutoReloader;
use sea_orm::DatabaseConnection;
use serde::Serialize;
use serde_json::json;
use std::any::Any;
use std::error::Error;
use std::fmt::Debug;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use tokio::net::{TcpListener, ToSocketAddrs};

#[derive(Debug)]
pub struct Env {
    pub env: String,
    pub debug: bool,
    pub vite_url: Option<String>,
}

impl Env {
    pub fn new(env: String, debug: bool, vite_url: Option<String>) -> Env {
        Env {
            env,
            debug,
            vite_url,
        }
    }
}

type AppState = Box<dyn Any + Send + Sync>;

pub struct App {
    router: Arc<Router>,
    listener: TcpListener,
    template: AutoReloader,
    db: Option<DatabaseConnection>,
    env: Env,
    logger: Logger,
    pub state: AppState,
}

impl App {
    pub async fn new<A: ToSocketAddrs>(
        router: Router,
        addr: A,
        template: AutoReloader,
        db: DatabaseConnection,
        logger: Logger,
        env: Env,
        state: AppState,
    ) -> App {
        App {
            router: Arc::new(router),
            listener: TcpListener::bind(addr).await.unwrap(),
            template,
            db: Some(db),
            env,
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
            vite_url => self.env.vite_url,
            ..context
        };
        self.template
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
        self.env.env == "production"
    }

    pub fn is_local(&self) -> bool {
        self.env.env == "local"
    }

    fn error(&self, error: &HttpError, json: bool) -> Result<Response<Full<Bytes>>, HttpError> {
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

/// Future executor that utilises `tokio` threads.
#[non_exhaustive]
#[derive(Default, Debug, Clone)]
pub struct TokioExecutor;

impl TokioExecutor {
    pub fn new() -> Self {
        Self {}
    }
}

impl<Fut> Executor<Fut> for TokioExecutor
where
    Fut: Future + Send + 'static,
    Fut::Output: Send + 'static,
{
    fn execute(&self, fut: Fut) {
        tokio::spawn(fut);
    }
}

pub async fn run(app: Arc<App>) -> Result<(), Box<dyn Error + Send + Sync>> {
    loop {
        let (stream, _) = app.listener().accept().await?;
        let io = TokioIo::new(stream);
        let app = Arc::clone(&app);

        tokio::task::spawn(async move {
            if let Err(err) = auto::Builder::new(TokioExecutor::new())
                .serve_connection(
                    io,
                    service_fn(move |request: Request<Incoming>| {
                        handle_request(Arc::clone(&app), request)
                    }),
                )
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn handle_request(
    app: Arc<App>,
    request: Request<Incoming>,
) -> Result<Response<Full<Bytes>>, HttpError> {
    let wants_json = if let Some(accepts) = request.headers().get("Accept") {
        accepts == "application/json"
    } else {
        false
    };

    // Catch panics within app.dispatch()
    match AssertUnwindSafe(app.dispatch(request)).catch_unwind().await {
        Ok(option) => match option {
            Some(result) => {
                if let Err(err) = &result {
                    return app.error(err, wants_json);
                }

                result
            }
            None => app.error(
                &HttpError::new(
                    404,
                    if wants_json {
                        "Endpoint not found."
                    } else {
                        "Page not found."
                    }
                    .to_owned(),
                ),
                wants_json,
            ),
        },
        Err(error) => {
            // Caught panic. Send details if app is local and debug is enabled
            let msg = if let Some(msg) = error.downcast_ref::<&str>() {
                *msg
            } else if let Some(msg) = error.downcast_ref::<String>() {
                msg
            } else {
                "Unknown panic."
            };

            app.error(&HttpError::new(500, msg.to_string()), false)
        }
    }
}
