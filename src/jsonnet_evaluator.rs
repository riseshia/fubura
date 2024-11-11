use std::{collections::HashSet, path::Path};

use serde_json::Value;

use crate::cli::StrKeyVal;

use rsjsonnet_front::Session;
use rsjsonnet_lang::program::Value as JsonnetValue;

pub fn eval(file_path: &str, ext_str: &[StrKeyVal]) -> Result<Value, String> {
    let source_path = Path::new(file_path);

    let arena = rsjsonnet_lang::arena::Arena::new();
    let mut session = Session::new(&arena);

    let mut ext_names = HashSet::new();

    for arg in ext_str.iter() {
        let key = session.program().intern_str(&arg.var);
        if !ext_names.insert(key) {
            let err_msg = format!("External variable {:?} defined more than once", arg.var);
            return Err(err_msg);
        }

        let val = if let Some(val) = &arg.val {
            JsonnetValue::string(val.as_ref())
        } else {
            JsonnetValue::null()
        };

        let thunk = session.program_mut().value_to_thunk(&val);
        session.program_mut().add_ext_var(key, &thunk);
    }

    let Some(thunk) = session.load_real_file(source_path) else {
        return Err("Failed to load file".to_string());
    };

    let Some(value) = session.eval_value(&thunk) else {
        return Err("Failed to evaluate file".to_string());
    };

    let Some(json_str) = session.manifest_json(&value, true) else {
        return Err("Failed to marshal as json".to_string());
    };

    let json: Value = serde_json::from_str(&json_str).unwrap();

    Ok(json)
}
