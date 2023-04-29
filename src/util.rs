use js_sys::{Object, Reflect};
use std::env;
use wasm_bindgen::JsValue;

pub fn flatten_json(nested_json: Object) -> Object {
    let mut flat_json = Object::new();

    fn flatten(path_prefix: String, node: Object, flat_json: &mut Object) {
        let keys = Object::keys(&node);
        for key in keys.iter() {
            let key = key.as_string().unwrap();
            let content_or_node = Reflect::get(&node, &key.clone().into()).unwrap();
            let joined_path = path_join(vec![path_prefix.clone(), key.clone()]);
            if content_or_node.is_string() {
                Reflect::set(&flat_json, &joined_path.into(), &content_or_node).unwrap();
            } else if content_or_node.is_object() && !content_or_node.is_null() {
                let content_or_node = Object::from(content_or_node);
                if Object::keys(&content_or_node).length() > 0 {
                    flatten(joined_path, content_or_node, flat_json);
                }
            } else {
                Reflect::set(&flat_json, &joined_path.into(), &JsValue::null()).unwrap();
            }
        }
    }

    flatten(String::from(""), nested_json, &mut flat_json);

    flat_json
}

pub fn path_join(paths: Vec<String>) -> String {
    let mut path = String::new();
    for part in paths {
        path = format!("{}/{}", path, part);
    }
    path_normalize(path)
}

pub fn path_relative(from: String, to: String, cwd: Option<String>) -> String {
    let cwd = cwd.unwrap_or(process_cwd());
    let to = path_resolve(
        vec![if to == "" { cwd.clone() } else { to }],
        Some(cwd.clone()),
    );
    let from = path_resolve(
        vec![if from == "" { cwd.clone() } else { from }],
        Some(cwd.clone()),
    );
    if from == to {
        return String::from("");
    }
    let from = filename_to_steps(from, Some(cwd.clone()));
    let to = filename_to_steps(to, Some(cwd.clone()));
    let mut length = 0;
    let mut same = true;
    for i in 0..from.len() {
        if from[i] != to[i] {
            same = false;
            break;
        }
        length += 1;
    }
    if same {
        return to[length..].join("/");
    }
    let mut up = from.len() - length;
    let mut down = Vec::new();
    while up > 0 {
        down.push("..".to_string());
        up -= 1;
    }
    down.extend_from_slice(&to[length..]);
    down.join("/")
}

pub fn filename_to_steps(filename: String, base: Option<String>) -> Vec<String> {
    let full_path = path_resolve(vec![filename], base);
    let full_path_sans_slash = full_path.trim_start_matches("/");
    if full_path_sans_slash == "" {
        return vec![];
    }
    full_path_sans_slash
        .split("/")
        .map(|s| s.to_string())
        .collect()
}

pub fn path_resolve(paths: Vec<String>, cwd: Option<String>) -> String {
    let cwd = cwd.unwrap_or(process_cwd());
    assert!(cwd.starts_with("/"), "cwd must be absolute");
    if paths.len() == 0 {
        return path_normalize(cwd);
    }
    let mut path = String::new();
    for part in paths.iter().rev() {
        let normalized_part = path_normalize(part.clone());
        path = format!("{}/{}", normalized_part, path);
        if normalized_part.starts_with("/") {
            break;
        }
    }
    if !path.starts_with("/") {
        path = format!("{}/{}", cwd, path);
    }
    path_normalize(path)
}

pub fn path_normalize(path: String) -> String {
    let mut normalized_path = Vec::new();
    if path == "." || path == ".." {
        return path;
    }
    for step in path.split("/") {
        if step == "." || step == "" {
            continue;
        }
        if step == ".." {
            normalized_path.pop();
            continue;
        }
        normalized_path.push(step);
    }
    let mut normalized_path = normalized_path.join("/");
    if normalized_path == "" {
        normalized_path = String::from("/");
    }
    if path.starts_with("/") {
        normalized_path = format!("/{}", normalized_path);
    }
    normalized_path
}

pub fn process_cwd() -> String {
    let mut cwd = env::var("CWD").unwrap_or(env::var("PWD").unwrap_or(String::from("")));
    if cwd == "" {
        eprintln!("current working directory not set, falling back to '/'.");
        eprintln!("set either CWD or PWD environment variables.");
        cwd = String::from("/");
    }
    cwd
}

#[test]
fn test_path_relative() {
    assert_eq!(
        path_relative(
            String::from("/data/orandea/test/aaa"),
            String::from("/data/orandea/impl/bbb"),
            None
        ),
        String::from("../../impl/bbb")
    );
}

#[test]
fn test_path_resolve() {
    assert_eq!(
        path_resolve(
            vec![
                String::from("/foo"),
                String::from("/bar"),
                String::from("baz")
            ],
            None
        ),
        String::from("/bar/baz")
    );

    assert_eq!(
        path_resolve(
            vec![
                String::from("/foo"),
                String::from("/bar"),
                String::from("baz"),
                String::from(".."),
                String::from("qux"),
                String::from("."),
                String::from("quux")
            ],
            None
        ),
        String::from("/bar/qux/quux")
    );

    assert_eq!(
        path_resolve(vec![String::from("/foo/bar/.././baz"),], None),
        String::from("/foo/baz")
    );

    assert_eq!(
        path_resolve(
            vec![String::from("foo/bar/.././baz"),],
            Some(String::from("/qux"))
        ),
        String::from("/qux/foo/baz")
    );
}
