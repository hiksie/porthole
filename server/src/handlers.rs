use axum::body::Body;
use axum::extract::{Multipart, Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio_util::io::ReaderStream;

use crate::state::AppState;

#[derive(Serialize)]
pub struct FolderDto {
    id: String,
    name: String,
}

/// GET /api/folders
pub async fn list_folders(State(state): State<AppState>) -> impl IntoResponse {
    let roots: Vec<FolderDto> = state
        .folders()
        .iter()
        .map(|folder| FolderDto {
            id: folder.id.clone(),
            name: folder.name.clone(),
        })
        .collect();

    axum::Json(roots)
}

#[derive(Deserialize)]
pub struct BrowseQuery {
    root: String,
    #[serde(default)]
    path: String,
}

#[derive(Serialize)]
pub struct EntryDto {
    name: String,
    path: String,
    is_dir: bool,
    size: u64,
}

/// GET /api/browse?root=<id>&path=<relative>
pub async fn browse(
    State(state): State<AppState>,
    Query(query): Query<BrowseQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let root = state.root_by_id(&query.root).ok_or(StatusCode::NOT_FOUND)?;
    let dir = resolve_existing(&root.path, &query.path)?;
    if !dir.is_dir() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut entries = Vec::new();
    let mut read_dir = tokio::fs::read_dir(&dir)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let Ok(metadata) = entry.metadata().await else {
            continue;
        };
        let name = entry.file_name().to_string_lossy().into_owned();
        let rel_path = if query.path.is_empty() {
            name.clone()
        } else {
            format!("{}/{name}", query.path.trim_end_matches('/'))
        };
        entries.push(EntryDto {
            name,
            path: rel_path,
            is_dir: metadata.is_dir(),
            size: metadata.len(),
        });
    }
    entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.cmp(&b.name)));

    Ok(axum::Json(entries))
}

#[derive(Deserialize)]
pub struct FileQuery {
    root: String,
    path: String,
}

/// GET /api/download?root=<id>&path=<relative>
pub async fn download(
    State(state): State<AppState>,
    Query(query): Query<FileQuery>,
) -> Result<Response, StatusCode> {
    let root = state.root_by_id(&query.root).ok_or(StatusCode::NOT_FOUND)?;

    let file_path = resolve_existing(&root.path, &query.path)?;

    if !file_path.is_file() {
        return Err(StatusCode::NOT_FOUND);
    }

    let file = tokio::fs::File::open(&file_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let body = Body::from_stream(ReaderStream::new(file));

    let filename = file_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "file".to_string());

    let mime = mime_guess::from_path(&file_path).first_or_octet_stream();

    let headers = [
        (header::CONTENT_TYPE, mime.to_string()),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\""),
        ),
    ];

    Ok((headers, body).into_response())
}

#[derive(Deserialize)]
pub struct UploadQuery {
    root: String,
    #[serde(default)]
    path: String,
}

/// POST /api/upload?root=<id>&path=<relative-dir>
pub async fn upload(
    State(state): State<AppState>,
    Query(query): Query<UploadQuery>,
    mut multipart: Multipart,
) -> Result<StatusCode, StatusCode> {
    let root = state.root_by_id(&query.root).ok_or(StatusCode::NOT_FOUND)?;
    let dir = resolve_existing(&root.path, &query.path)?;

    if !dir.is_dir() {
        return Err(StatusCode::BAD_REQUEST);
    }

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let Some(file_name) = field.file_name().map(str::to_string) else {
            continue;
        };

        let safe_name = sanitize_filename(&file_name);
        let dest = dir.join(&safe_name);
        let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;

        tokio::fs::write(&dest, &data)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(StatusCode::OK)
}

fn resolve_existing(root: &Path, rel: &str) -> Result<PathBuf, StatusCode> {
    if rel.contains("..") || Path::new(rel).is_absolute() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let root_canon = root.canonicalize().map_err(|_| StatusCode::NOT_FOUND)?;

    let candidate = if rel.is_empty() {
        root_canon.clone()
    } else {
        root_canon.join(rel)
    };

    let candidate_canon = candidate
        .canonicalize()
        .map_err(|_| StatusCode::NOT_FOUND)?;

    if !candidate_canon.starts_with(&root_canon) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(candidate_canon)
}

fn sanitize_filename(name: &str) -> String {
    Path::new(name)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .filter(|n| !n.is_empty() && n != "." && n != "..")
        .unwrap_or_else(|| "file".to_string())
}
