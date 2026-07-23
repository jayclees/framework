use crate::support::logger::Logger;
use std::backtrace::Backtrace;
use std::cell::RefCell;
use std::path::PathBuf;

// Thread local safe variable where we store the last backtrace
thread_local! {
    static LAST_BACKTRACE: RefCell<Option<Backtrace>> = RefCell::new(None)
}

pub fn register_panic_hook(root: PathBuf) {
    // Make sure pattern is in the top scope of this function so it's only compiled once.
    let logger = Logger::new(root.clone());
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        LAST_BACKTRACE.with(|backtrace| {
            logger.log(info.to_string());
            *backtrace.borrow_mut() = Some(Backtrace::force_capture());
        });

        default_hook(info);
    }));
}
