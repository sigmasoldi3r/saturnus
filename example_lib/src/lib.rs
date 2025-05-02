use saturnus_rt::export_mod;
use st_macros::module;

#[module]
pub mod test {

    use saturnus_rt::{
        mem::St,
        vm::{
            Result, StVm,
            types::{Any, IntoAny, Table},
        },
    };

    pub async fn sum(_: St<StVm>, args: Vec<Any>) -> Result<Any> {
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
}

export_mod!(test);
