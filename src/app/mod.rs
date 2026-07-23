pub mod app;
pub mod builder;

use crate::http::error::HttpError;
pub use app::App;
use futures::FutureExt;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::rt::Executor;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto;
use std::error::Error;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;

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
