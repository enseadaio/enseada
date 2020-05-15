use actix_web::{HttpRequest, HttpResponse, ResponseError};
use actix_web::web::{Data, Json, Path};

use crate::docker::error::{Error, ErrorCode};
use crate::docker::manifest::resolver::{ManifestResolver, ManifestResolve};
use crate::docker::manifest::s2::{ImageManifest, ManifestList};
use crate::docker::mime::IMAGE_MANIFEST_V2;
use crate::docker::{Result, Name, validate_name};
use crate::docker::handler::ImageNamePath;

pub async fn get(
    req: HttpRequest,
    resolver: Data<ManifestResolver>,
    path: Path<ImageNamePath>,
) -> HttpResponse {
    if !validate_name(&path.group, &path.name) {
        return Error::from(ErrorCode::NameInvalid).error_response();
    }

    let name = Name::new(path.group.clone(), path.name.clone());

    let accept = req.headers().get(http::header::ACCEPT)
        .map(|header| format!("{:?}", header))
        // FIXME: insomnia sends the header with quotes
        .and_then(|accept| if accept == "\"*/*\"" { None } else { Some(accept) })
        .unwrap_or_else(|| IMAGE_MANIFEST_V2.to_string());

    log::debug!("Received request for image manifest with type '{}'", accept);

    let res = match accept.as_ref() {
        IMAGE_MANIFEST_V2 => {
            let image = match resolver.resolve_image(&name).await {
                Ok(image) => image,
                Err(err) => return err.error_response(),
            };
            image.map(|image| {
                HttpResponse::Ok().json(image)
            })
        },
        _ => Some(Error::from(ErrorCode::Unsupported).error_response()),
    };

    match res {
        Some(res) => res,
        None => Error::from(ErrorCode::ManifestUnknown).error_response(),
    }
}