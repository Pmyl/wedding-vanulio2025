use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};
use wedding_vanulio2025::{Invite, InviteId};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
    let path = Some("/media/pmyl/Tardis/Projects/wedding-2025/api/invites");

    if let None = path {
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(().into())?);
    }

    let files = std::fs::read_dir(path.unwrap())?;

    for file in files {
        let file = file?;
        let file_name = file
            .file_name()
            .into_string()
            .expect("File name to be valid");
        println!("Processing file {}", file_name);

        // deserialize + serialize just to ensure the json is a valid invite
        let invite_file = std::fs::File::open(file.path())?;
        let invite: Invite = serde_json::from_reader(invite_file)?;
        println!("Invite deserialized");

        let Ok(id) = InviteId::new(file_name.clone()) else {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(format!("Invalid invite id {}", file_name).into())?);
        };

        invite.update_in_blob(&id).await?;
    }

    Ok(Response::builder()
        .status(StatusCode::CREATED)
        .body(().into())?)
}
