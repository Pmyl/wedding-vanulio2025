use serde_json::json;
use vercel_blob::client::{PutCommandOptions, VercelBlobApi, VercelBlobClient};
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
    let blob_client = VercelBlobClient::new();
    blob_client
        .put(
            "mystuff/stuff.txt",
            "some content",
            PutCommandOptions {
                content_type: Some("text/plain".to_string()),
                ..Default::default()
            },
        )
        .await?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(json!({ "message": "wedding!" }).to_string().into())?)
}
