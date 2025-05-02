pub mod mem;
pub mod native;
pub mod vm;

#[macro_export]
macro_rules! export_mod {
    ( $name:ident ) => {
        #[unsafe(no_mangle)]
        extern "stdcall" fn __load_saturnus_modules__() {
            //opaque: *mut std::ffi::c_void) {
            println!("Hey mf {}", stringify!($ident));
            // use saturnus_rt::{mem::St, vm::StVm};
            // unsafe {
            //     let vm: &St<StVm> = std::mem::transmute(opaque);
            //     let mut globals = vm.lock().get_globals();
            //     let mut modlib = {
            //         let key = vm.lock().create_string("__modules__");
            //         vm.lock().get_table(&globals, key)
            //     }
            //     .unwrap_table();
            //     {
            //         let key = vm.lock().create_string(stringify!($name));
            //         let value = ($name::load_mod(vm.clone()));
            //         vm.lock().set_table(&mut modlib, key, value)
            //     }
            // }
        }
    };
}
