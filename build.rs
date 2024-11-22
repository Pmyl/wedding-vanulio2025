use std::collections::HashMap;
use std::fs;
use std::io::Write;
use tera::{Context, Tera};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tera = Tera::new("templates/*.html").unwrap();

    let mut translations_json = serde_json::from_str::<HashMap<&str, serde_json::Value>>(
        include_str!("templates/translations.json"),
    )?;

    println!("English language");
    let translations_json_value = translations_json.remove("en").unwrap();
    let context = Context::from_value(translations_json_value)?;
    let rendered = tera.render("index.html", &context)?;
    std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("public/index.html")?
        .write_all(rendered.as_bytes())?;

    println!("Italian language");
    let translations_json_value = translations_json.remove("it").unwrap();
    let context = Context::from_value(translations_json_value)?;
    let rendered = tera.render("index.html", &context)?;
    fs::create_dir("public/it").ok();
    std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("public/it/index.html")?
        .write_all(rendered.as_bytes())?;

    Ok(())
}
