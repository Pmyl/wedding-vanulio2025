use std::{
    env,
    io::{BufWriter, Write},
    path::Path,
};

use uuid::Uuid;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};
use wedding_vanulio2025::{Invite, InviteId};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
    let path = env::var("PATH_TO_INVITES").expect("PATH_TO_INVITES env var to be set");
    let mut invites_output = BufWriter::new(
        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(Path::new(&path).join("invites_created.csv"))?,
    );

    for invite_result in get_invites_from_csv()? {
        let (invite, id) = invite_result?;
        invite.update_in_blob(&id).await?;

        invites_output.write(
            format!(
                "\"{}\",\"https://vanulio.fun/rsvp?_id={}\"\n",
                invite.name,
                id.as_str()
            )
            .as_bytes(),
        )?;
    }

    Ok(Response::builder()
        .status(StatusCode::CREATED)
        .body(().into())?)
}

// fn get_invites_from_dir() -> Result<impl Iterator<Item = Result<(Invite, InviteId), Error>>, Error>
// {
//     let path = env::var("PATH_TO_INVITES").expect("PATH_TO_INVITES env var to be set");
//     let files = std::fs::read_dir(Path::new(&path).join("invites"))?;

//     Ok(files.into_iter().map(|file| {
//         let file = file?;
//         let file_name = file
//             .file_name()
//             .into_string()
//             .expect("File name to be valid");
//         println!("Processing file {}", file_name);

//         // deserialize + serialize just to ensure the json is a valid invite
//         let invite_file = std::fs::File::open(file.path())?;
//         let invite: Invite = serde_json::from_reader(invite_file)?;
//         println!("Invite deserialized");

//         let id = InviteId::new(file_name.clone())?;

//         Ok((invite, id))
//     }))
// }

fn get_invites_from_csv() -> Result<impl Iterator<Item = Result<(Invite, InviteId), Error>>, Error>
{
    let path = env::var("PATH_TO_INVITES").expect("PATH_TO_INVITES env var to be set");
    let path = Path::new(&path).join("invites.csv");

    Ok(csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(std::fs::File::open(path)?)
        .into_deserialize()
        .map(|record| {
            let (name, guests, language) = record?;
            let id = InviteId::new(Uuid::new_v4().simple().to_string())?;
            Ok((
                Invite {
                    name,
                    responded: false,
                    attending: None,
                    notes: None,
                    guests,
                    vegetarians: 0,
                    language,
                },
                id,
            ))
        }))
}
