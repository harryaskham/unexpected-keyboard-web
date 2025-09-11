use tvix_eval::{EvaluationBuilder, Value, EvalIO, FileType, GlobalsMap, prepare_globals, SourceCode};
use wasm_bindgen::prelude::*;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use rustc_hash::FxHashMap;
use include_dir::{include_dir, Dir};
use std::io::{self, Cursor, Read};
use bytes::Bytes;

static NIX_LIB_DIR: Dir = include_dir!("nix_lib");

struct EmbeddedIO;

impl EvalIO for EmbeddedIO {
    fn path_exists(&self, path: &Path) -> io::Result<bool> {
        let relative_path = path.strip_prefix("/").unwrap_or(path);
        Ok(NIX_LIB_DIR.get_file(relative_path).is_some() || NIX_LIB_DIR.get_dir(relative_path).is_some())
    }

    fn open(&self, path: &Path) -> io::Result<Box<dyn Read>> {
        let relative_path = path.strip_prefix("/").unwrap_or(path);
        NIX_LIB_DIR.get_file(relative_path)
            .map(|file| Box::new(Cursor::new(file.contents())) as Box<dyn Read>)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "File not found"))
    }

    fn file_type(&self, path: &Path) -> io::Result<FileType> {
        let relative_path = path.strip_prefix("/").unwrap_or(path);
        if NIX_LIB_DIR.get_file(relative_path).is_some() {
            Ok(FileType::Regular)
        } else if NIX_LIB_DIR.get_dir(relative_path).is_some() {
            Ok(FileType::Directory)
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "File not found"))
        }
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<(Bytes, FileType)>> {
        let relative_path = path.strip_prefix("/").unwrap_or(path);
        let dir = NIX_LIB_DIR.get_dir(relative_path)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Directory not found"))?;

        let mut entries = Vec::new();
        for entry in dir.entries() {
            let file_type = if entry.as_file().is_some() { FileType::Regular } else { FileType::Directory };
            entries.push((Bytes::from(entry.path().file_name().unwrap().to_str().unwrap().to_string()), file_type));
        }
        Ok(entries)
    }

    fn import_path(&self, path: &Path) -> io::Result<PathBuf> {
        // For now, we'll just return the path itself, as we're not dealing with a Nix store
        Ok(path.to_path_buf())
    }

    fn store_dir(&self) -> Option<String> {
        None
    }
}

fn indent(level: usize) -> String {
    "  ".repeat(level)
}

fn pretty_print_value(value: &Value, indent_level: usize) -> String {
    match value {
        Value::Attrs(attrs) => {
            let mut s = String::new();
            s.push_str(&format!("{}{{\n", indent(indent_level)));
            for (name, val) in attrs.iter_sorted() {
                s.push_str(&format!("{}{} = {}\n", indent(indent_level + 1), name.ident_str(), pretty_print_value(val, indent_level + 1)));
            }
            s.push_str(&format!("{}}}", indent(indent_level)));
            s
        }
        Value::List(list) => {
            let mut s = String::new();
            s.push_str(&format!("{}[
", indent(indent_level)));
            for val in list.iter() {
                s.push_str(&format!("{}{}\n", indent(indent_level + 1), pretty_print_value(val, indent_level + 1)));
            }
            s.push_str(&format!("{}]", indent(indent_level)));
            s
        }
        _ => value.to_string(),
    }
}

#[wasm_bindgen]
pub fn tvix_eval(expr: &str) -> String {
    let io_handle = Rc::new(EmbeddedIO);
    let source_code = SourceCode::default();

    // Create a base GlobalsMap with builtins
    let base_globals = prepare_globals(vec![], vec![], source_code.clone(), true);

    // Insert the evaluated lib into the main globals_map
    let main_globals_vec: Vec<(&'static str, Value)> = vec![
        ("lib", Value::Path(Box::new(PathBuf::from("/lib")))),
    ];
    let main_globals = prepare_globals(main_globals_vec, vec![], source_code.clone(), true);

    let evaluation = EvaluationBuilder::new(io_handle.clone())
        .with_globals(main_globals)
        .build();

    let result = evaluation.evaluate(expr, Some(PathBuf::from("/dummy/path/to/code.nix").parent().unwrap().to_path_buf()));

    if result.errors.is_empty() {
        if let Some(value) = result.value {
            pretty_print_value(&value, 0)
        } else {
            "(no value)".to_string()
        }
    } else {
        let error_messages: Vec<String> = result.errors.into_iter().map(|err| {
            let mut current_err = &err;
            let mut messages = vec![];

            loop {
                messages.push(format!("Kind: {:?}, Span: {:?}", current_err.kind, current_err.span));

                if let tvix_eval::ErrorKind::NativeError { err: nested_err, .. } = &current_err.kind {
                    current_err = nested_err.as_ref();
                } else {
                    break;
                }
            }
            messages.join("\n")
        }).collect();
        error_messages.join("\n")
    }
}
