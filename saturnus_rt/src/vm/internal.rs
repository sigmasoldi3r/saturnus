use mlua::Lua;

use super::StVm;

/// This trait is an escape hatch from the saturnus virtual machine abstraction,
/// in case that you need to access the internal Lua runtime.
///
/// This should only be accessed in case of very specific needs, and in case you
/// need to do some patch.
///
/// Otherwise, VM access is preferred.
pub trait InternalRuntime {
    fn access_internal_runtime(&self) -> &Lua;
}

impl InternalRuntime for StVm {
    fn access_internal_runtime(&self) -> &Lua {
        &self.rt
    }
}
