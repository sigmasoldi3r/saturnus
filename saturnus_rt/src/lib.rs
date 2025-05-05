pub mod mem;
pub mod native;
pub mod vm;

#[macro_export]
macro_rules! saturnus_export {
    ( $mod_name:ident; $($name:ident),* ) => {
        #[unsafe(no_mangle)]
        extern "C" fn __saturnus_module_symbols__() -> (String, Vec<String>) {
            (
                stringify!($mod_name).to_string(),
                vec![ $(stringify!($name).to_string()),* ]
            )
        }
    };
}
