#[macro_export]
macro_rules! dd {
    ( $( $x:expr ),* ) => {
        dbg!($($x)*);
        panic!("Dumping and dying.");
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
        {
            let root = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
            let logger = $crate::support::logger::Logger::new(root);
            $(
                logger.log_line($x, $crate::helper::get_line!().as_str());
            )*
        }
    };
}

pub use log;
