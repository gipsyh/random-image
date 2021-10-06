#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
use rand::prelude::SliceRandom;
use rocket::http::ContentType;
use rocket::Response;
use std::fs;
use std::io::{Cursor, Read};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::{fs::File, vec};

#[derive(Clone)]
struct ImageData(Arc<Vec<u8>>);

#[derive(Clone)]
struct Image {
    format: ContentType,
    data: ImageData,
}

impl AsRef<[u8]> for ImageData {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

lazy_static::lazy_static! {
    static ref IMAGES: RwLock<Vec<Image>> = RwLock::new(Vec::new());
}

#[get("/image")]
fn files<'a>() -> Response<'a> {
    let image = IMAGES
        .read()
        .unwrap()
        .choose(&mut rand::thread_rng())
        .unwrap()
        .clone();
    Response::build()
        .header(image.format)
        .sized_body(Cursor::new(image.data))
        .finalize()
}

fn add_file(path: PathBuf) -> Result<(), ()> {
    if !path.exists() {
        return Err(());
    }
    if let Some(ext) = path.extension() {
        if let Some(ext) = ContentType::from_extension(&ext.to_string_lossy()) {
            if ext == ContentType::JPEG || ext == ContentType::PNG {
                let mut file = File::open(path).unwrap();
                let mut file_buf = Vec::new();
                file.read_to_end(&mut file_buf).unwrap();
                IMAGES.write().unwrap().push(Image {
                    format: ext,
                    data: ImageData(Arc::new(file_buf)),
                });
            }
        }
    }
    Err(())
}

fn main() {
    for entry in fs::read_dir("images").unwrap() {
        let _ = add_file(entry.unwrap().path());
    }
    rocket::ignite().mount("/", routes![files]).launch();
}
