use serde::Deserialize;
use vercel_runtime::{run, Body, Error, Request, RequestPayloadExt, Response, StatusCode};
use wedding_vanulio2025::{FromBlobError, Invite, InviteId};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

#[derive(Deserialize, Debug)]
struct AttendingRequest {
    #[serde(rename = "_id")]
    id: String,
    guests: u16,
    vegetarians: u16,
    notes: String,
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    let request = req.payload::<AttendingRequest>()?.unwrap();

    let Ok(id) = InviteId::new(request.id) else {
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .body("Incomplete url".into())?);
    };

    let mut invite = match Invite::from_blob(&id).await {
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

    if invite.responded {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "text/html")
            .body("Already responded".into())?);
    }

    invite.attending = Some(true);
    invite.responded = true;
    invite.guests = request.guests;
    invite.vegetarians = request.vegetarians;
    invite.notes = if request.notes.is_empty() {
        None
    } else {
        Some(request.notes)
    };

    invite.update_in_blob(&id).await?;
    invite.notify().await?;

    Ok(Response::builder().status(StatusCode::OK).body(().into())?)
}
