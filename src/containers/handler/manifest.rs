use actix_web::{HttpRequest, HttpResponse, Responder, ResponseError};
use actix_web::web::{Data, Path};
use http::header;

use crate::containers::error::{Error, ErrorCode};
use crate::containers::handler::NameParams;
use crate::containers::manifest::resolver::{ManifestResolve, ManifestResolver};
use crate::containers::mime::oci::v1::{IMAGE_INDEX as OCI_IMAGE_INDEX_V1, IMAGE_MANIFEST as OCI_IMAGE_MANIFEST_V1};
use crate::containers::name::{is_valid_name, Name};

pub async fn get(
    req: HttpRequest,
    resolver: Data<ManifestResolver>,
    path: Path<NameParams>,
) -> HttpResponse {
    let name = Name::from(path.into_inner());
    if !is_valid_name(name.group(), name.name()) {
        return Error::from(ErrorCode::NameInvalid).error_response();
    }

    let accept = req.headers().get(header::ACCEPT)
        .and_then(|header| header.to_str().ok())
        .and_then(|accept| if accept == "*/*" { None } else { Some(accept) })
        .unwrap_or_else(|| OCI_IMAGE_INDEX_V1);

    log::debug!("Received request for image manifest with type '{}'", accept);

    let res = match accept {
        OCI_IMAGE_INDEX_V1 => {
            let list = match resolver.resolve_list(&name).await {
                Ok(image) => image,
                Err(err) => return err.error_response(),
            };
            list.map(|list| {
                HttpResponse::Ok().json(list)
            })
        },
        OCI_IMAGE_MANIFEST_V1 => {
            let image = match resolver.resolve_image(&name).await {
                Ok(image) => image,
                Err(err) => return err.error_response(),
            };
            image.map(|image| {
                HttpResponse::Ok().json(image)
            })
        },
        _ => Some(Error::from(ErrorCode::Unsupported).error_response())
    };

    match res {
        Some(res) => res,
        None => Error::from(ErrorCode::NameUnknown).error_response(),
    }
}