use crate::build_router;
use crate::state::AppState;
use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use http_body_util::BodyExt;
use tower::ServiceExt;

fn setup_shared_folder() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("hello.txt"), b"Hello upload").unwrap();
    std::fs::create_dir(dir.path().join("sub")).unwrap();
    std::fs::write(dir.path().join("sub").join("nested.txt"), b"nested").unwrap();
    dir
}

fn shared_router() -> (tempfile::TempDir, Router, String) {
    let dir = setup_shared_folder();
    let state = AppState::new(vec![dir.path().to_path_buf()]);
    let root = state.root_ids().into_iter().next().expect("one root");
    (dir, build_router(state), root)
}

async fn body_json(response: Response) -> serde_json::Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).expect("valid json body")
}

#[tokio::test]
async fn list_roots_returns_configured_folder() {
    let (_dir, router, root) = shared_router();

    let response = router
        .oneshot(Request::get("/api/folders").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let json = body_json(response).await;
    assert_eq!(json[0]["id"], root);
    assert!(json[0]["name"].as_str().unwrap().len() > 0);
}

#[tokio::test]
async fn folder_id_is_stable_across_rebuilds() {
    let dir = setup_shared_folder();
    let first = AppState::new(vec![dir.path().to_path_buf()]).root_ids();
    let second = AppState::new(vec![dir.path().to_path_buf()]).root_ids();
    assert_eq!(first, second);
}

#[tokio::test]
async fn browse_lists_entries_sorted_dirs_first() {
    let (_dir, router, root) = shared_router();

    let response = router
        .oneshot(
            Request::get(format!("/api/browse?root={root}&path="))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let json = body_json(response).await;
    let entries = json.as_array().unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0]["name"], "sub");
    assert_eq!(entries[0]["is_dir"], true);
    assert_eq!(entries[1]["name"], "hello.txt");
    assert_eq!(entries[1]["is_dir"], false);
}

#[tokio::test]
async fn download_returns_file_contents() {
    let (_dir, router, root) = shared_router();

    let response = router
        .oneshot(
            Request::get(format!("/api/download?root={root}&path=hello.txt"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&bytes[..], b"Hello upload");
}

#[tokio::test]
async fn download_from_nested_folder_works() {
    let (_dir, router, root) = shared_router();

    let response = router
        .oneshot(
            Request::get(format!("/api/download?root={root}&path=sub/nested.txt"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&bytes[..], b"nested");
}

#[tokio::test]
async fn path_traversal_outside_root_is_rejected() {
    let (_dir, router, root) = shared_router();

    let response = router
        .oneshot(
            Request::get(format!("/api/download?root={root}&path=../../etc/passwd"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn unknown_root_id_returns_not_found() {
    let (_dir, router, _root) = shared_router();

    let response = router
        .oneshot(
            Request::get("/api/browse?root=deadbeefdeadbeef&path=")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn upload_writes_file_into_shared_folder() {
    let (dir, router, root) = shared_router();

    let boundary = "TESTBOUNDARY";
    let body = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"uploaded.txt\"\r\n\
         Content-Type: text/plain\r\n\r\n\
         uploaded content\r\n\
         --{boundary}--\r\n"
    );

    let response = router
        .oneshot(
            Request::post(format!("/api/upload?root={root}&path="))
                .header(
                    "content-type",
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let saved = std::fs::read_to_string(dir.path().join("uploaded.txt")).unwrap();
    assert_eq!(saved, "uploaded content");
}

#[tokio::test]
async fn upload_sanitizes_path_traversal_in_filename() {
    let (dir, router, root) = shared_router();

    let boundary = "TESTBOUNDARY";
    let body = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"../evil.txt\"\r\n\
         Content-Type: text/plain\r\n\r\n\
         gotcha\r\n\
         --{boundary}--\r\n"
    );

    let response = router
        .oneshot(
            Request::post(format!("/api/upload?root={root}&path="))
                .header(
                    "content-type",
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    assert!(dir.path().join("evil.txt").exists());
    assert!(!dir.path().parent().unwrap().join("evil.txt").exists());
}

#[tokio::test]
async fn folder_update_is_visible_without_restart() {
    let dir = setup_shared_folder();
    let (handle, state) = crate::FolderHandle::new(&[]);
    let router = build_router(state.clone());

    let response = router
        .oneshot(Request::get("/api/folders").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(body_json(response).await.as_array().unwrap().len(), 0);

    handle.update(&[dir.path().to_path_buf()]);

    let response = build_router(state)
        .oneshot(Request::get("/api/folders").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(body_json(response).await.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn root_serves_embedded_index_html() {
    let router = build_router(AppState::new(vec![]));

    let response = router
        .oneshot(Request::get("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(html.contains("<title>"));
}

#[tokio::test]
async fn app_js_is_served_with_its_own_content() {
    let router = build_router(AppState::new(vec![]));

    let response = router
        .oneshot(Request::get("/app.js").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let js = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(js.contains("openRootList"));
}
