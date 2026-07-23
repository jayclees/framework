use std::any::Any;
use super::app::{App, AppState};
use crate::env::Env;
use crate::routing::router::Router;
use crate::support::logger::Logger;
use crate::template::reloader;
use minijinja_autoreload::AutoReloader;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::{TcpListener, ToSocketAddrs};
use crate::database::db;

pub struct Builder<A: 'static + ToSocketAddrs> {
    root: PathBuf,
    env: Option<Env>,
    // router: Option<Arc<Router>>,
    router: Option<Router>,
    listen_addr: Option<A>,
    template: Option<AutoReloader>,
    // db: Option<Pin<Box<dyn Future<Output=Result<DatabaseConnection, Box<dyn Error + Send + Sync>>>>>>,
    db: Option<bool>,
    state: Option<AppState>,
}

impl<A: 'static + ToSocketAddrs> Builder<A> {
    pub fn new(root: PathBuf) -> Builder<A> {
        Builder {
            root,
            env: Some(Env::init()),
            router: None,
            listen_addr: None,
            template: None,
            db: Some(false),
            state: None,
        }
    }

    pub fn router(&mut self, router: Router) -> &mut Builder<A> {
        self.router = Some(router);
        self
    }

    pub fn listen(&mut self, addr: A) -> &mut Builder<A> {
        self.listen_addr = Some(addr);
        self
    }

    pub fn template(&mut self) -> &mut Builder<A> {
        self.template = Some(reloader(self.root.clone()));
        self
    }

    pub fn db(&mut self) -> &mut Builder<A> {
        self.db = Some(true);
        self
    }

    pub fn state<S: Any + Send + Sync>(&mut self, state: Box<S>) -> &mut Builder<A> {
        self.state = Some(state);
        self
    }

    pub async fn build(&mut self) -> App {
        let env = self.env.take().unwrap();
        let db = if let Some(enabled) = self.db && enabled {
            Some(db(&env).await.unwrap())
        } else {
            None
        };

        App::init(
            env,
            Arc::new(self.router.take().unwrap()),
            TcpListener::bind(self.listen_addr.take().unwrap()).await.unwrap(),
            self.template.take(),
            db,
            Logger::new(self.root.clone()),
            self.state.take().unwrap(),
        )
    }
}
