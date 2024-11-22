use std::collections::HashMap;
use std::fs;
use std::io::Write;
use tera::{Context, Tera};

fn main() {
    let tera = Tera::new("templates/*.html").unwrap();

    let mut translations_json = serde_json::from_str::<HashMap<&str, serde_json::Value>>(
        include_str!("templates/context.json"),
    )
    .unwrap();

    let translations_json_value = translations_json.remove("en").unwrap();
    let context = Context::from_value(translations_json_value).unwrap();
    let rendered = tera.render("index.html", &context).unwrap();
    std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("public/index.html")
        .unwrap()
        .write_all(rendered.as_bytes())
        .unwrap();

    let translations_json_value = translations_json.remove("it").unwrap();
    let context = Context::from_value(translations_json_value).unwrap();
    let rendered = tera.render("index.html", &context).unwrap();
    fs::create_dir("public/it").ok();
    std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("public/it/index.html")
        .unwrap()
        .write_all(rendered.as_bytes())
        .unwrap();

    println!("cargo:rerun-if-changed=templates");
}
