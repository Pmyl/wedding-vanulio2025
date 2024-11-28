use std::error::Error;

use bytes::Buf;
use serde::{Deserialize, Serialize};
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
            "Invite uploaded to file {} with name {}",
            put_result.url, put_result.pathname
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
