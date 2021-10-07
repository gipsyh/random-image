#[macro_use]
extern crate rocket;
use httpdate::HttpDate;
use rocket::http::hyper::header::{IF_MODIFIED_SINCE, LAST_MODIFIED};
use rocket::http::{ContentType, Header, Status};
use rocket::request::{self, FromRequest, Outcome};
use rocket::response::Redirect;
use rocket::response::Responder;
use rocket::{uri, Request, Response};
use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug)]
struct IfModifiedSince(Option<HttpDate>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for IfModifiedSince {
    type Error = ();
    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let date = match req.headers().get_one(IF_MODIFIED_SINCE.as_str()) {
            Some(str) => HttpDate::from_str(str).ok(),
            None => None,
        };
        Outcome::Success(IfModifiedSince(date))
    }
}

#[derive(Debug)]
struct Image {
    file: File,
    contenttype: ContentType,
    ifmodifiedsince: IfModifiedSince,
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for Image {
    fn respond_to(self, _request: &'r Request<'_>) -> rocket::response::Result<'static> {
        let lastmodified: HttpDate = self.file.metadata().unwrap().modified().unwrap().into();
        let lastmodifiedheader = Header::new(LAST_MODIFIED.to_string(), lastmodified.to_string());
        match self.ifmodifiedsince.0 {
            Some(ifmodifiedsince) if ifmodifiedsince >= lastmodified => Response::build()
                .status(Status::new(304))
                .header(lastmodifiedheader)
                .ok(),
            _ => Response::build()
                .header(lastmodifiedheader)
                .header(self.contenttype)
                .sized_body(None, tokio::fs::File::from(self.file))
                .ok(),
        }
    }
}

impl Image {
    async fn new(path: PathBuf, ifmodifiedsince: IfModifiedSince) -> io::Result<Image> {
        let file = File::open(&path)?;
        let ext = path
            .extension()
            .and_then(OsStr::to_str)
            .ok_or(io::ErrorKind::AddrInUse)?;
        let contenttype = ContentType::from_extension(ext).unwrap();
        Ok(Self {
            file,
            contenttype,
            ifmodifiedsince,
        })
    }
}

#[get("/image/<file>")]
async fn image<'a>(file: &str, ifmodifiedsince: IfModifiedSince) -> Option<Image> {
    let path = Path::new("images").join(file);
    Image::new(path, ifmodifiedsince).await.ok()
}

#[get("/image")]
fn random_image() -> Redirect {
    Redirect::to(uri!(image("1.jpeg")))
}

#[rocket::launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![image, random_image])
}
