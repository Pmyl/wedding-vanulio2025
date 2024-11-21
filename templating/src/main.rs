use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use tera::{Context, Tera};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tera = Tera::new("templating/src/templates/*.html").unwrap();

    let mut translations_json = serde_json::from_str::<HashMap<&str, serde_json::Value>>(
        include_str!("../../templating/src/templates/translations.json"),
    )?;

    let translations_json_value = translations_json.remove("en").unwrap();
    let context = Context::from_value(translations_json_value)?;
    let rendered = tera.render("index.html", &context)?;
    File::create("public/index.html")?.write_all(rendered.as_bytes())?;

    let translations_json_value = translations_json.remove("it").unwrap();
    let context = Context::from_value(translations_json_value)?;
    let rendered = tera.render("index.html", &context)?;
    fs::create_dir("public/it")?;
    File::create("public/it/index.html")?.write_all(rendered.as_bytes())?;

    Ok(())
}
