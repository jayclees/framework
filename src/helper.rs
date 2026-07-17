#[macro_export]
macro_rules! dd {
    ( $( $x:expr ),* ) => {
        dbg!($($x)*);
        std::process::exit(1);
    };
}

pub use dd;

#[macro_export]
macro_rules! get_line {
    () => {
        format!("{}:{}:{}", file!(), line!(), column!())
    };
}

pub use get_line;

#[macro_export]
macro_rules! log {
    ( $( $x:expr ),* ) => {
        let t1 = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let t2 = framework::support::logger::Logger::new(t1);
        $(
            t2.log($x);
        )*
        // let t1 = std::path::PathBuf::new(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    };
}

pub use log;
