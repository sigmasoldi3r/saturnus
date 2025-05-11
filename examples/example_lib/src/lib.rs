use runtime::saturnus_export;

use runtime::{
    mem::St,
    vm::{
        Result, StVm,
        types::{Any, IntoAny},
    },
};

#[unsafe(no_mangle)]
pub fn sum(_: St<StVm>, args: Vec<Any>) -> Result<Any> {
    let mut total = 0.0;
    for part in args.into_iter() {
        match part {
            Any::Integer(val) => total += val as f64,
            Any::Decimal(val) => total += val,
            Any::Boolean(val) => {
                total += match val {
                    true => 1.0,
                    false => 0.0,
                }
            }
            _ => (),
        }
    }
    Ok(total.into_any())
}

saturnus_export!(example; sum);
