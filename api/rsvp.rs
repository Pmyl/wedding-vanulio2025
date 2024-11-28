use std::collections::HashMap;
use tera::{Context, Tera};
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};
use wedding_vanulio2025::{FromBlobError, Invite, InviteId, Language};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    let mut query_params: HashMap<String, String> = req
        .uri()
        .query()
        .map(|v| {
            url::form_urlencoded::parse(v.as_bytes())
                .into_owned()
                .collect()
        })
        .unwrap_or_else(HashMap::new);
    println!("Query params {:?}", query_params);

    let Some(id) = query_params.remove("_id") else {
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .body("Invalid url".into())?);
    };

    let Ok(id) = InviteId::new(id) else {
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .body("Incomplete url".into())?);
    };

    println!("Id {:?}", id);

    let invite = match Invite::from_blob(&id).await {
        Ok(i) => i,
        Err(err) => match err {
            FromBlobError::Error(error) => Err(error)?,
            FromBlobError::NotFound => {
                return Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(().into())?)
            }
        },
    };

    let mut context = Context::new();
    context.insert("name", &invite.name);
    context.insert("responded", &invite.responded);
    context.insert("guests", &invite.guests);
    context.insert("vegetarians", &invite.vegetarians);
    context.insert("attending", &invite.attending);
    context.insert("notes", &invite.notes);

    let mut context_json = serde_json::from_str::<HashMap<&str, serde_json::Value>>(include_str!(
        "rsvp/context.json"
    ))?;
    let context_json_value = context_json
        .remove(match invite.language {
            Language::Italian => "it",
            Language::English => "en",
        })
        .unwrap();
    context.extend(Context::from_value(context_json_value)?);

    let rendered =
        Tera::one_off(include_str!("rsvp/rsvp-template.html"), &context, true).expect("asd");

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html")
        .body(rendered.into())?)
}
