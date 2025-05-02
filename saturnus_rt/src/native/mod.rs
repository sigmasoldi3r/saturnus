use st_macros::module;

#[module]
pub mod net {

    use reqwest::Method;

    use crate::{
        mem::St,
        table_get, table_set,
        vm::{
            Result, StVm,
            types::{Any, IntoAny, Table},
        },
    };

    pub async fn request(vm: St<StVm>, args: Vec<Any>) -> Result<Any> {
        let options: Table = args[0].clone().unwrap_table();
        let url = table_get!(vm; options, "url").unwrap_str().to_string();
        let get = vm.lock().create_string("get");
        let method = table_get!(vm; options, "method")
            .or(get)
            .unwrap_str()
            .to_string()
            .to_lowercase();
        let method = Method::from_bytes(method.as_bytes()).unwrap();
        let cb = args[1].clone().unwrap_function();
        let cl = reqwest::Client::new();
        let mut req = cl.request(method, url);
        if let Any::Str(body) = table_get!(vm; options, "body") {
            let body = body.to_string();
            req = req.body(body);
        }
        let res = req
            .send()
            .await
            .map_err(|err| crate::vm::StError::Internal(Box::new(err)))?;
        let out = res
            .text()
            .await
            .map_err(|err| crate::vm::StError::Internal(Box::new(err)))?;
        let out = vm.lock().create_string(out);
        let mut res = vm
            .lock()
            .create_table()
            .map_err(|err| crate::vm::StError::Internal(Box::new(err)))?;
        table_set!(vm; res, "body" => out);
        vm.lock().invoke(cb, vec![res.into_any()])
    }
}
