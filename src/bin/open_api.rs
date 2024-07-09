use std::fs::File;
use std::io::Write;

use utoipa::OpenApi;
use rust_jwks::http::{MyErrorResponse, MyResponse};

#[derive(OpenApi)]
#[openapi(
    info(description = "Rust Service Example API Docs"),
    components(schemas(MyResponse, MyErrorResponse)),
    paths(
        rust_jwks::http::public,
        rust_jwks::http::secure
    )
)]
pub struct ApiDoc;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let file_path = args.get(1).map(|s| s.as_str()).unwrap_or("./docs/swagger.json");
    let file_path = std::path::Path::new(file_path);
    if let Some(prefix) = file_path.parent() {
        std::fs::create_dir_all(prefix)?;
    }

    let mut file = File::create(file_path)?;
    file.write_all(ApiDoc::openapi().to_json()?.as_bytes())?;

    Ok(())
}
