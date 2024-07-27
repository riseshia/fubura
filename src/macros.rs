/// Allows you to exit immediately with error message.
/// It is a shortcut for `eprintln!` followed by `std::process::exit(1)`.
///
/// # Examples
///
/// ```no_run
/// # use fubura::fast_exit;
/// let x = 42;
/// if x != 42 {
///   fast_exit!("x is not 42");
/// } else {
///   println!("x is 42");
/// }
/// ```
#[macro_export]
macro_rules! fast_exit {
    () => {
        {
            std::process::exit(1);
        }
    };
    ($msg:expr) => {
        {
            eprintln!("{}", $msg);
            std::process::exit(1);
        }
    };
    ($fmt:expr, $($arg:tt)*) => {
        {
            eprintln!($fmt, $($arg)*);
            std::process::exit(1);
        }
    };
}
