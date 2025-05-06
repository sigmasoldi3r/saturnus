use st_macros::module;

#[module]
pub mod net {

    use std::time::Duration;

    use reqwest::Method;

    use crate::{
        mem::St,
        table, table_get, table_set,
        vm::{
            Result, StError, StVm,
            types::{Any, IntoAny, Table},
        },
    };

    pub async fn request(vm: StVm, args: Vec<Any>) -> Result<Any> {
        let Any::Table(options) = args[0].clone() else {
            return Err(StError::type_error("First argument must be a table"));
        };
        let Any::Function(func) = args[1].clone() else {
            return Err(StError::type_error("Second argument must be a function"));
        };
        vm.spawn({
            let vm = vm.clone();
            async move {
                let url = table_get!(vm; options, "url").unwrap_str().to_string();
                let get = vm.create_string("GET");
                let method = table_get!(vm; options, "method")
                    .or(get)
                    .unwrap_str()
                    .to_string()
                    .to_uppercase();
                let method = Method::from_bytes(method.as_bytes()).unwrap();
                let cl = reqwest::Client::new();
                let mut req = cl.request(method, url);
                if let Any::Str(body) = table_get!(vm; options, "body") {
                    let body = body.to_string();
                    req = req.body(body);
                }
                let res = req.send().await.unwrap();
                let headers = res.headers();
                let body = res.text().await.unwrap();
                vm.invoke(
                    func,
                    vec![
                        table! { vm;
                            "body" => body,
                        }
                        .into_any(),
                    ],
                )
                .unwrap();
            }
        })
        .await;

        Ok(Any::unit())
    }
}

#[module]
pub mod json {
    use serde_json::Value;

    use crate::{
        mem::St,
        table, tuple,
        vm::{
            Result, StVm,
            types::{Any, IntoAny, IntoSaturnus, Table},
        },
    };

    trait ToSt {
        fn into_saturnus(self, vm: StVm) -> Any;
    }
    impl ToSt for Vec<Any> {
        fn into_saturnus(self, vm: StVm) -> Any {
            let mut tbl = vm.create_table().unwrap();
            let mut k = 0;
            for item in self.into_iter() {
                k += 1;
                tbl.set(k, item);
            }
            Any::Table(tbl)
        }
    }
    impl ToSt for Value {
        fn into_saturnus(self, vm: StVm) -> Any {
            match self {
                Value::Null => Any::unit(),
                Value::Bool(v) => Any::Boolean(v),
                Value::Number(number) => {
                    if number.is_i64() {
                        Any::Integer(number.as_i64().unwrap())
                    } else {
                        Any::Decimal(number.as_f64().unwrap())
                    }
                }
                Value::String(v) => vm.create_string(v.as_bytes()).into_saturnus(),
                Value::Array(values) => values
                    .into_iter()
                    .map(|value| value.into_saturnus(vm.clone()))
                    .collect::<Vec<_>>()
                    .into_saturnus(vm.clone()),
                Value::Object(map) => {
                    let mut tbl = vm.create_table().unwrap();
                    for (k, v) in map.into_iter() {
                        tbl.set(k, v.into_saturnus(vm.clone()));
                    }
                    Any::Table(tbl)
                }
            }
        }
    }

    pub async fn stringify(vm: StVm, args: Vec<Any>) -> Result<Any> {
        todo!()
    }

    pub async fn parse(vm: StVm, args: Vec<Any>) -> Result<Any> {
        use serde_json::{Result, Value};
        let Any::Str(first) = &args[0] else {
            return Ok(tuple!(vm; Any::unit(), "First argument must be a string").into_any());
        };
        let first = first.to_string();
        let Ok(out): Result<Value> = serde_json::from_str(&first) else {
            return Ok(tuple!(vm; Any::unit(), "Invalid json string").into_any());
        };
        Ok(tuple!(vm; out.into_saturnus(vm), Any::unit()).into_any())
    }
}
