pub use tracing as __tracing;
pub use tracing::{debug, info, log, trace};

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::log::__tracing::warn!(
            file=file!(),
            line=line!(),
            column=column!(),
            $($arg)*
        );
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::log::__tracing::error!(
            file=file!(),
            line=line!(),
            column=column!(),
            $($arg)*
        )
    };
}

pub fn init() {
    // install global subscriber configured based on RUST_LOG envvar.
    tracing_subscriber::fmt::init();
}
