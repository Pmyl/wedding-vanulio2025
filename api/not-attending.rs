use serde::Deserialize;
use vercel_runtime::{run, Body, Error, Request, RequestPayloadExt, Response, StatusCode};
use wedding_vanulio2025::{FromBlobError, Invite};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

#[derive(Deserialize, Debug)]
struct NotAttendingRequest {
    #[serde(rename = "_id")]
    id: String,
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    let request = req.payload::<NotAttendingRequest>()?.unwrap();

    if request.id.len() != 32 {
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .body("Incomplete url".into())?);
    };

    let mut invite = match Invite::from_blob(&request.id).await {
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

    invite.attending = Some(false);
    invite.responded = true;
    invite.update_in_blob(&request.id).await.expect("test");

    Ok(Response::builder().status(StatusCode::OK).body(().into())?)
}