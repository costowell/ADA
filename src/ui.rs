use std::io::Cursor;

use image::ImageReader;
use log::error;
use slint::{Image, SharedPixelBuffer};

use crate::{drink::DrinkUser, runtime::runtime};

slint::include_modules!();

async fn image_from_uid(uid: &str) -> Result<Image, anyhow::Error> {
    let rt = runtime();
    let url = format!("https://profiles.csh.rit.edu/image/{}", uid);
    let bytes = rt
        .spawn(async move {
            reqwest::get(url).await.unwrap().bytes().await.unwrap()
        })
        .await?;
    let img = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()?
        .decode()?;
    Ok(Image::from_rgb8(SharedPixelBuffer::clone_from_slice(
        &img.to_rgb8(),
        img.width(),
        img.height(),
    )))
}

impl AppWindow {
    pub async fn login(&self, user: DrinkUser) {
        self.set_logged_in(true);
        self.set_uid(user.uid.clone().into());
        self.set_credits(user.drink_balance.try_into().unwrap());
        match image_from_uid(user.uid.as_str()).await {
            Ok(image) => self.set_profile_picture(image),
            Err(err) => error!("Failed to fetch profile picture: {err}"),
        }
    }
}
