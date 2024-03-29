use std::{
    fs::{create_dir_all, File},
    io::BufWriter,
};

use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::args::DownloadCommand;

#[derive(Debug, Deserialize, Serialize)]
pub struct Post {
    pub id: i32,
    pub score: i32,
    pub rating: char,
    pub file_ext: String,
    pub file_url: Option<String>,
    pub large_file_url: Option<String>,
}

impl Post {
    pub fn download(self, client: &Client, args: &DownloadCommand) -> Result<()> {
        let is_webm = self
            .large_file_url
            .as_ref()
            .ok_or(anyhow!("No url detected"))?
            .contains(".webm");

        let file_extension = if &self.file_ext == "zip" && is_webm {
            "webm"
        } else {
            &self.file_ext
        };

        let url = if is_webm {
            self.large_file_url.ok_or(anyhow!("No url detected"))?
        } else {
            self.file_url.ok_or(anyhow!("No url detected"))?
        };

        #[rustfmt::skip]
        let subfolder = match self.rating {
            's' => "sensitive",
            'q' => "questionable",
            'e' => "explicit",
            'g' => "general",
            _   => "unknown",
        };

        let sub_folder_path = args.save_location.join(subfolder);

        if !sub_folder_path.exists() {
            create_dir_all(&sub_folder_path)?;
        }

        let filename = format!("{}_{}.{}", &self.score, &self.id, file_extension);

        let file_path = sub_folder_path.join(filename);

        if file_path.exists() {
            return Ok(());
        }

        let mut response = client.get(url).send()?;

        let mut file = BufWriter::new(File::create(&file_path)?);

        response.copy_to(&mut file)?;

        Ok(())
    }
}
