#[macro_export]
macro_rules! base_log {
    ( $label:literal, $colour:ident, $str:literal $(, $arg:expr )* ) => {
        {
            use colored::Colorize as _;
            println!("{} {}: {}", format!("{}", module_path!()).dimmed(), format!($label).$colour(), format!($str $(, $arg)*));
        }
    };
}

#[macro_export]
macro_rules! info {
    ( $str:literal $(, $arg:expr )* ) => {
        base_log!("info", cyan, $str $(, $arg )*)
    };
}

#[macro_export]
macro_rules! warn {
    ( $str:literal $(, $arg:expr )* ) => {
        base_log!("warning", yellow, $str $(, $arg )*)
    };
}
