use crate::support::logger::Logger;
use std::backtrace::Backtrace;
use std::cell::RefCell;

// Thread local safe variable where we store the last backtrace
thread_local! {
    static LAST_BACKTRACE: RefCell<Option<Backtrace>> = RefCell::new(None)
}

pub fn register_panic_hook(logger: Logger) {
    // Make sure pattern is in the top scope of this function so it's only compiled once.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        LAST_BACKTRACE.with(|backtrace| {
            logger.log(info.to_string());
            *backtrace.borrow_mut() = Some(Backtrace::force_capture());
        });

        default_hook(info);
    }));
}
