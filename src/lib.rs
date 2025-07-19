use tvix_eval::{EvaluationBuilder, DummyIO};
use wasm_bindgen::prelude::*;
use std::rc::Rc;
use std::path::PathBuf;

#[wasm_bindgen]
pub fn tvix_eval(expr: &str) -> String {
    let evaluation = EvaluationBuilder::new_pure().build();
    let result = evaluation.evaluate(expr, Some(PathBuf::from("/dummy/path/to/code.nix")));

    if result.errors.is_empty() {
        format!("{:?}", result.value)
    } else {
        format!("Error: {:?}", result.errors)
    }
}