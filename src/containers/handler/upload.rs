use actix_web::{HttpRequest, HttpResponse, ResponseError};
use actix_web::web::{Bytes, Data, Path, Query};
use http::header;
use serde::Deserialize;

use crate::containers::digest::Digest;
use crate::containers::error::{Error, ErrorCode};
use crate::containers::handler::NameParams;
use crate::containers::header::BLOB_UPLOAD_ID;
use crate::containers::name::Name;
use crate::containers::upload::{UploadChunk, UploadService};

#[derive(Debug, Deserialize)]
pub struct UploadParams {
    upload_id: String,
}

pub async fn start(
    path: Path<NameParams>,
    uploads: Data<UploadService>,
    req: HttpRequest,
) -> HttpResponse {
    let image = Name::from(path.into_inner());
    let group = image.group().to_string();
    let name = image.name().to_string();
    log::info!("Starting upload for image {}", &image);
    let upload = uploads.start_upload(image).await;
    match upload {
        Ok(upload) => {
            log::info!("Upload {} started", upload.id().id());
            let redirect = req.url_for("upload", &[&group, &name, upload.id().id()]).unwrap();
            HttpResponse::Accepted()
                .header(header::LOCATION, redirect.to_string())
                .header(header::RANGE, format!("bytes=0-{}", upload.latest_offset()))
                .finish()
        }
        Err(err) => err.error_response(),
    }
}

pub async fn get_status(
    path: Path<NameParams>,
    upload: Path<UploadParams>,
    uploads: Data<UploadService>,
    req: HttpRequest,
) -> HttpResponse {
    let image = Name::from(path.into_inner());
    let group = image.group().to_string();
    let name = image.name().to_string();
    let upload_id = &upload.upload_id;
    log::debug!("Getting status for upload {}", upload_id);
    let res = uploads.find_upload(upload_id).await
        .and_then(|opt| opt.ok_or_else(|| Error::from(ErrorCode::BlobUploadUnknown)));
    match res {
        Ok(upload) => {
            let redirect = req.url_for("upload", &[&group, &name, upload.id().id()]).unwrap();
            HttpResponse::NoContent()
                .header(header::LOCATION, redirect.to_string())
                .header(header::RANGE, format!("bytes=0-{}", upload.latest_offset()))
                .finish()
        }
        Err(err) => err.error_response(),
    }
}

pub async fn upload_chunk(
    name: Path<NameParams>,
    upload: Path<UploadParams>,
    uploads: Data<UploadService>,
    req: HttpRequest,
    body: Bytes,
) -> HttpResponse {
    let image = Name::from(name.into_inner());
    let upload_id = &upload.upload_id;
    let chunk = match UploadChunk::from_request(req.headers(), body) {
        Ok(chunk) => chunk,
        Err(err) => return err.error_response(),
    };

    match uploads.push_chunk(upload_id, chunk).await {
        Ok(upload) => {
            let upload_id = upload.id().id();
            let redirect = req.url_for("upload", &[image.group(), image.name(), upload_id]).unwrap();
            HttpResponse::Accepted()
                .header(header::LOCATION, redirect.to_string())
                .header(header::RANGE, format!("bytes=0-{}", upload.latest_offset()))
                .header(BLOB_UPLOAD_ID, upload_id)
                .finish()
        }
        Err(err) => err.error_response(),
    }
}

pub async fn complete(
    name: Path<NameParams>,
    upload: Path<UploadParams>,
    query: Query<DigestParams>,
    uploads: Data<UploadService>,
    req: HttpRequest,
    body: Bytes,
) -> HttpResponse {
    let image = Name::from(name.into_inner());
    let upload_id = &upload.upload_id;
    let digest = &query.digest;
    log::debug!("Completing upload {} for layer {}", upload_id, digest);

    let chunk = match UploadChunk::from_request(req.headers(), body) {
        Ok(chunk) => chunk,
        Err(err) => return err.error_response(),
    };

    let redirect = req.url_for("blob", &[image.group(), image.name(), &digest.to_string()]).unwrap();
    match uploads.complete_upload(upload_id, digest, chunk).await {
        Ok(_) => HttpResponse::Created()
            .header(header::LOCATION, redirect.to_string())
            .finish(),
        Err(err) => err.error_response(),
    }
}

pub async fn cancel(
    name: Path<NameParams>,
    upload: Path<UploadParams>,
    uploads: Data<UploadService>,
) -> HttpResponse {
    let upload_id = &upload.upload_id;
    log::debug!("Cancelling upload {}", upload_id);

    let res = uploads.find_upload(upload_id).await
        .and_then(|opt| opt.ok_or_else(|| Error::from(ErrorCode::BlobUploadUnknown)));

    let upload = match res {
        Ok(upload) => upload,
        Err(err) => return err.error_response(),
    };

    match uploads.delete_upload(upload).await {
        Ok(()) => HttpResponse::Ok().finish(),
        Err(err) => err.error_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct DigestParams {
    digest: Digest,
}

pub fn exists(
    path: Path<NameParams>,
    digest: Path<DigestParams>,
) -> HttpResponse {
    let name = Name::from(path.into_inner());
    let digest = &digest.digest;
    log::debug!("Checking existence of layer {} for image {}", digest, name);
    HttpResponse::NotImplemented().finish()
}
