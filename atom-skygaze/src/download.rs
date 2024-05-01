use axum::{
    body::Body,
    extract::Query,
    http::{header, HeaderMap, StatusCode},
    response::Response,
};
use serde::Deserialize;
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use axum::response::IntoResponse;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct FileQuery {
    filename: String,
}

pub async fn download_file(query: Query<FileQuery>) -> Result<Response<Body>, StatusCode> {
    let filename = &query.filename;

    let file_path = PathBuf::from(format!("/media/mmc/records/detected/{}", filename));

    if !file_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    let file = File::open(&file_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", file_path.file_name().unwrap().to_str().unwrap())
            .parse()
            .unwrap(),
    );
    headers.insert(header::CONTENT_TYPE, "application/octet-stream".parse().unwrap());

    Ok((headers, body).into_response())
}
