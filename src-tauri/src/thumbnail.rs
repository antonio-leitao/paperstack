use super::{database, document_library};
use hayro::hayro_interpret::InterpreterSettings;
use hayro::hayro_syntax::Pdf;
use hayro::{render, RenderCache, RenderSettings};
use serde::Serialize;
use std::{
    io::Cursor,
    panic::{catch_unwind, AssertUnwindSafe},
    path::{Path, PathBuf},
};
use tauri::{AppHandle, Emitter};

// Target width of a card thumbnail, in pixels (a little headroom over the ~240px
// card). Bump THUMBNAIL_VERSION whenever the size/format changes so old files
// are treated as stale and regenerated.
const THUMBNAIL_WIDTH: u32 = 300;
const THUMBNAIL_VERSION: &str = "v1";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ThumbnailReady {
    document_id: String,
}

// $APPDATA/thumbnails/<content-hash>.<version>.jpg — keyed by PDF content so it
// dedupes and survives re-import, and version-tagged so a format change is a
// clean regeneration.
// Resolves the thumbnail directory without touching the filesystem.
pub(crate) fn thumbnails_directory(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(database::app_data_directory_path(app)?.join("thumbnails"))
}

// Names a thumbnail inside an already-resolved directory, so a bulk load can
// resolve the directory once instead of once per row.
pub(crate) fn thumbnail_file_in(directory: &Path, content_hash: &str) -> PathBuf {
    directory.join(format!("{content_hash}.{THUMBNAIL_VERSION}.jpg"))
}

// Where a thumbnail lives, without touching the filesystem. Reading does not
// need the directory to exist.
pub(crate) fn thumbnail_file(app: &AppHandle, content_hash: &str) -> Result<PathBuf, String> {
    Ok(thumbnail_file_in(&thumbnails_directory(app)?, content_hash))
}

// Same, but creates the thumbnail directory. Use this only before writing one.
pub(crate) fn thumbnail_path(app: &AppHandle, content_hash: &str) -> Result<PathBuf, String> {
    let directory = database::app_data_directory(app)?.join("thumbnails");
    std::fs::create_dir_all(&directory)
        .map_err(|error| format!("Could not create the thumbnail directory: {error}"))?;
    Ok(directory.join(format!("{content_hash}.{THUMBNAIL_VERSION}.jpg")))
}

// Renders the first page to a JPEG and stores it (best-effort). Returns Ok(true)
// when a thumbnail was written, Ok(false) when one already existed. All PDF
// rendering happens in pure Rust via hayro — there is no native library.
pub(crate) fn ensure_thumbnail(
    app: &AppHandle,
    content_hash: &str,
    pdf_bytes: &[u8],
) -> Result<bool, String> {
    let path = thumbnail_path(app, content_hash)?;
    if path.exists() {
        return Ok(false);
    }
    let jpeg = render_first_page_jpeg(pdf_bytes)?;
    let temporary = path.with_extension("jpg.tmp");
    std::fs::write(&temporary, &jpeg)
        .map_err(|error| format!("could not write thumbnail: {error}"))?;
    std::fs::rename(&temporary, &path).map_err(|error| {
        let _ = std::fs::remove_file(&temporary);
        format!("could not finalize thumbnail: {error}")
    })?;
    Ok(true)
}

fn render_first_page_jpeg(pdf_bytes: &[u8]) -> Result<Vec<u8>, String> {
    // hayro is comprehensive but still labelled experimental, so isolate a panic
    // on an unusual PDF: a single bad file should leave the card image-less, not
    // break import.
    let bytes = pdf_bytes.to_vec();
    catch_unwind(AssertUnwindSafe(move || render_first_page(bytes)))
        .map_err(|_| "PDF rendering panicked".to_string())?
}

fn render_first_page(pdf_bytes: Vec<u8>) -> Result<Vec<u8>, String> {
    // ---- hayro API touch points (confirm against `cargo build`) -------------
    // Pdf::new(impl Into<PdfData>), pdf.pages() (derefs to &[Page], so .first()),
    // hayro::render(page, &RenderCache, &InterpreterSettings, &RenderSettings),
    // and the vello Pixmap accessors below.
    let pdf = Pdf::new(pdf_bytes).map_err(|error| format!("could not parse PDF: {error:?}"))?;
    let pages = pdf.pages();
    let page = pages.first().ok_or_else(|| "PDF has no pages".to_string())?;

    let settings = RenderSettings {
        x_scale: 1.0,
        y_scale: 1.0,
        width: None,
        height: None,
        ..Default::default()
    };
    let pixmap = render(
        page,
        &RenderCache::new(),
        &InterpreterSettings::default(),
        &settings,
    );

    let width = u32::from(pixmap.width());
    let height = u32::from(pixmap.height());
    if width == 0 || height == 0 {
        return Err("rendered page was empty".to_string());
    }

    // Composite the premultiplied RGBA pixels over a white background into RGB.
    // For premultiplied source over opaque white: out = src + (255 - alpha).
    let source = pixmap.data_as_u8_slice();
    let mut rgb = vec![0u8; (width * height * 3) as usize];
    for (pixel, out) in source.chunks_exact(4).zip(rgb.chunks_exact_mut(3)) {
        let background = 255 - u32::from(pixel[3]);
        out[0] = (u32::from(pixel[0]) + background).min(255) as u8;
        out[1] = (u32::from(pixel[1]) + background).min(255) as u8;
        out[2] = (u32::from(pixel[2]) + background).min(255) as u8;
    }

    let buffer = image::RgbImage::from_raw(width, height, rgb)
        .ok_or_else(|| "could not build image buffer".to_string())?;
    let target_height = ((u64::from(THUMBNAIL_WIDTH) * u64::from(height)) / u64::from(width))
        .max(1) as u32;
    let resized = image::imageops::resize(
        &buffer,
        THUMBNAIL_WIDTH,
        target_height,
        image::imageops::FilterType::Triangle,
    );

    let mut jpeg = Vec::new();
    image::DynamicImage::ImageRgb8(resized)
        .write_to(&mut Cursor::new(&mut jpeg), image::ImageFormat::Jpeg)
        .map_err(|error| format!("could not encode thumbnail: {error}"))?;
    Ok(jpeg)
}

// On startup, generate a thumbnail for any document that doesn't yet have a
// current one (imported before this feature) and tell open windows when each is
// ready. Runs in a background thread; sequential, so rendering never piles up.
pub(crate) fn recover_missing_thumbnails(app: AppHandle) {
    let documents = match document_library::all_document_hashes(&app) {
        Ok(documents) => documents,
        Err(_) => return,
    };
    for (document_id, content_hash) in documents {
        let Ok(path) = thumbnail_file(&app, &content_hash) else {
            continue;
        };
        if path.exists() {
            continue;
        }
        let Ok(pdf_path) = document_library::document_file(&app, &document_id) else {
            continue;
        };
        let Ok(bytes) = std::fs::read(&pdf_path) else {
            continue;
        };
        if let Ok(true) = ensure_thumbnail(&app, &content_hash, &bytes) {
            let _ = app.emit("thumbnail-ready", ThumbnailReady { document_id });
        }
    }
}
