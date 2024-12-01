use bytes::Buf;
use mail_send::{mail_builder::MessageBuilder, SmtpClientBuilder};
use serde::{Deserialize, Serialize};
use std::{env, error::Error};
use vercel_blob::{
    client::{
        DelCommandOptions, DownloadCommandOptions, PutCommandOptions, VercelBlobApi,
        VercelBlobClient,
    },
    error::VercelBlobError,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Invite {
    pub name: String,
    pub responded: bool,
    pub attending: Option<bool>,
    pub notes: Option<String>,
    pub guests: u16,
    pub vegetarians: u16,
    pub language: Language,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Language {
    Italian,
    English,
}

#[derive(Debug)]
pub struct InviteId(String);

impl InviteId {
    pub fn new(id: String) -> Result<Self, Box<dyn Error + Send + Sync>> {
        if id.len() != 32 {
            return Err("Invite id must be 32 characters long".into());
        }

        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Invite {
    pub async fn update_in_blob(&self, id: &InviteId) -> Result<(), Box<dyn Error + Send + Sync>> {
        let invite_serialized = serde_json::to_string(&self)?;
        println!("Invite re-serialized");

        let blob_client = VercelBlobClient::new();
        let blobs = blob_client
            .list(vercel_blob::client::ListCommandOptions {
                prefix: Some(format!("invites/{}", id.as_str())),
                ..Default::default()
            })
            .await?;

        if blobs.blobs.len() > 1 {
            return Err("Two of same invite exists".into());
        }

        if blobs.blobs.len() == 1 {
            blob_client
                .del(&blobs.blobs[0].url, DelCommandOptions::default())
                .await?;
            println!("Found previous version of invite, removed");
        }

        let put_result = blob_client
            .put(
                &format!("invites/{}", id.as_str()),
                invite_serialized,
                PutCommandOptions {
                    content_type: Some("application/json".to_string()),
                    ..Default::default()
                },
            )
            .await?;
        println!(
            "Invite for {} uploaded to file {} with name {}",
            self.name, put_result.url, put_result.pathname
        );

        Ok(())
    }

    pub async fn from_blob(id: &InviteId) -> Result<Self, FromBlobError> {
        let blob_client = VercelBlobClient::new();

        let blobs = blob_client
            .list(vercel_blob::client::ListCommandOptions {
                prefix: Some(format!("invites/{}", id.as_str())),
                ..Default::default()
            })
            .await?;

        if blobs.blobs.len() > 1 {
            return Err(FromBlobError::Error("Two of same invite exists".into()));
        }

        if blobs.blobs.len() == 0 {
            println!("Invite not found with id {}", id.as_str());
            return Err(FromBlobError::NotFound);
        }

        println!("Invite exists");

        let invite_compressed = blob_client
            .download(&blobs.blobs[0].url, DownloadCommandOptions::default())
            .await?;
        println!("Invite downloaded");

        let invite: Invite = serde_json::from_reader(invite_compressed.reader())?;
        println!("Invite deserialized");

        Ok(invite)
    }

    pub async fn notify(self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let Some(attending) = self.attending else {
            return Err("Invite not responded".into());
        };

        println!("Sending notification");

        let vanulio_email =
            env::var("VANULIO_EMAIL").expect("Expect VANULIO_EMAIL env variable to be set");
        let giulio_email =
            env::var("GIULIO_EMAIL").expect("Expect GIULIO_EMAIL env variable to be set");
        let vanulio_user =
            env::var("VANULIO_USER").expect("Expect VANULIO_USER env variable to be set");
        let vanulio_password =
            env::var("VANULIO_PASSWORD").expect("Expect VANULIO_PASSWORD env variable to be set");

        let mut message = MessageBuilder::new()
            .from(("Vanulio", vanulio_email.as_ref()))
            .to(vec![
                ("Giulio", giulio_email.as_ref()),
                ("Vanulio", vanulio_email.as_ref()),
            ]);

        if attending {
            message = message
                .subject(format!("YES RSVP {}", self.name))
                .html_body(format!(
                    "{} has {} guests attending the wedding.<br />
                Vegetarians: {}<br />
                Notes: {}",
                    self.name,
                    self.guests,
                    self.vegetarians,
                    tera::escape_html(&self.notes.unwrap_or_default())
                ));
        } else {
            message = message
                .subject(format!("NO RSVP {}", self.name))
                .html_body(format!("{} has declined the invite :(", self.name));
        }

        SmtpClientBuilder::new("smtp.gmail.com", 587)
            .implicit_tls(false)
            .credentials((vanulio_user.as_ref(), vanulio_password.as_ref()))
            .connect()
            .await?
            .send(message)
            .await?;

        println!("Notification sent to {}", vanulio_email);
        println!("Notification sent to {}", giulio_email);
        Ok(())
    }
}

pub enum FromBlobError {
    Error(Box<dyn Error + Sync + Send>),
    NotFound,
}

impl From<VercelBlobError> for FromBlobError {
    fn from(err: VercelBlobError) -> Self {
        FromBlobError::Error(err.into())
    }
}

impl From<serde_json::Error> for FromBlobError {
    fn from(err: serde_json::Error) -> Self {
        FromBlobError::Error(err.into())
    }
}
