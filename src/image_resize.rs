use reqwest::*;
use std::string::String;
use image::*;

pub async fn download_and_resize_image(src: String, width: String, quality: String) -> Buffer {
  let img_bytes = reqwest::blocking::get(String::from(&src))?
  .bytes()?;

  let image = image::load_from_memory(&img_bytes)?;
}