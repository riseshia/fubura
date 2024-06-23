use std::path::Path;

use serde_json::Value;

use crate::cli::StrKeyVal;

use rsjsonnet_lang::program::Value as JsonnetValue;

pub fn eval(file_path: &str, ext_str: &[StrKeyVal]) -> Result<Value, String> {
    let source_path = Path::new(file_path);

    let mut session = rsjsonnet_front::Session::new();

    ext_str.iter().for_each(|ext_str| {
        let key = session.program().str_interner().intern(&ext_str.var);
        let val = if let Some(val) = &ext_str.val {
            JsonnetValue::string(val.as_ref())
        } else {
            JsonnetValue::null()
        };
        let value_thunk = session.program_mut().value_to_thunk(&val);
        session.program_mut().add_ext_var(key, &value_thunk);
    });

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
    println!("{}", json);

    Ok(json)
}
