use crate::digest::Digest;

pub fn chunk_key(upload_id: &str, chunk_offset: usize) -> String {
    format!(
        "artifacts/oci/uploads/{}/chunks/{}",
        upload_id, chunk_offset
    )
}

pub fn blob_key(digest: &Digest) -> String {
    format!("artifacts/oci/blobs/{}", digest.to_string())
}
