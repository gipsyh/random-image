#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

use std::io::{Cursor, Read};
use std::sync::{Arc, RwLock};
use std::{fs::File, vec};

use rand::prelude::SliceRandom;
use rocket::http::ContentType;
use rocket::Response;

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

fn init() {
    let mut file = File::open("images/ttt.png").unwrap();
    let mut file_buf = Vec::new();
    file.read_to_end(&mut file_buf).unwrap();
    IMAGES.write().unwrap().push(Image {
        format: ContentType::JPEG,
        data: ImageData(Arc::new(file_buf)),
    });
}

fn main() {
    init();
    rocket::ignite().mount("/", routes![files]).launch();
}

#[test]
fn test() {
    use std::fs;
    for entry in fs::read_dir("images").unwrap() {
        let entry = entry.unwrap();
        // let a =NamedFile::open("aaa").unwrap();
        println!("{:?}", entry.metadata());
    }
}
