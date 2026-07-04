use std::ffi::CString;
use std::os::raw::c_int;
use std::panic;
use std::sync::Once;

use anyhow::Result;
use rayon::prelude::*;
use serde_json::Value;

use crate::content_store::ContentStore;
use crate::settings::Settings;

static INSTALL_QUIET_PANIC_HOOK: Once = Once::new();

/// pdf-extract panics noisily on malformed PDFs and prints a multi-page stack
/// dump to stderr. We catch the panic, so installing a no-op hook silences
/// the spew without losing control flow.
fn install_quiet_panic_hook() {
    INSTALL_QUIET_PANIC_HOOK.call_once(|| {
        panic::set_hook(Box::new(|_| {}));
    });
}

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
pub struct ExtractPdfTextArgs {
    /// Re-extract even if text already exists for this document.
    #[arg(long, default_value_t = false)]
    pub force: bool,
}

/// Run text extraction on every PDF document in the content store, writing
/// results into the `document_text` table. PDF parsing can panic on malformed
/// inputs, so each call is wrapped in `catch_unwind` and logged on failure.
pub fn run(settings: &Settings, args: &ExtractPdfTextArgs) -> Result<()> {
    install_quiet_panic_hook();
    // pdf-extract emits log::warn! on benign issues (glyph-name vs unicode
    // ligature mismatches, unknown glyph names) for every PDF; silence them.
    log::set_max_level(log::LevelFilter::Error);

    let store = ContentStore::new(settings);
    let pdfs = store.list_pdf_documents()?;
    let total = pdfs.len();
    println!("Extracting text from {total} PDF document(s)");

    // Phase 1: resolve candidates — skip docs that already have text (unless --force)
    // and those with no backing PDF.
    let mut candidates: Vec<(String, String)> = Vec::new(); // (hkap_id, media_id)
    let mut skipped = 0usize;
    let mut no_pdf = 0usize;
    let mut malformed = 0usize;
    for (hkap_id, content_json) in pdfs {
        if !args.force && store.has_document_text(&hkap_id).unwrap_or(false) {
            skipped += 1;
            continue;
        }
        match media_cloud_file_id_from_metadata(&content_json) {
            MediaIdLookup::Found(id) => candidates.push((hkap_id, id)),
            MediaIdLookup::Null => no_pdf += 1,
            MediaIdLookup::Malformed => malformed += 1,
        }
    }
    println!(
        "{} to extract ({} skipped, {} no_pdf, {} malformed)",
        candidates.len(), skipped, no_pdf, malformed
    );

    // Phase 2: process in chunks — fetch PDF bytes (sequential DB reads), extract
    // text in parallel across all cores (the CPU bottleneck), then batch-upsert.
    let mut ok = 0usize;
    let mut failed = 0usize;
    let mut missing_media = 0usize;
    const CHUNK: usize = 64;
    for chunk in candidates.chunks(CHUNK) {
        let mut with_bytes: Vec<(String, Vec<u8>)> = Vec::with_capacity(chunk.len());
        for (hkap_id, media_id) in chunk {
            match store.get_media(media_id) {
                Ok(b) => with_bytes.push((hkap_id.clone(), b)),
                Err(_) => missing_media += 1,
            }
        }
        let texts: Vec<(String, String)> = with_bytes
            .par_iter()
            .filter_map(|(hkap_id, bytes)| {
                extract_pdf_text(bytes).map(|t| (hkap_id.clone(), normalize_whitespace(&t)))
            })
            .collect();
        failed += with_bytes.len() - texts.len();
        ok += texts.len();
        store.upsert_document_texts(texts)?;
        println!("  {}/{} extracted...", ok, candidates.len());
    }

    println!(
        "Done. extracted={ok} skipped={skipped} no_pdf={no_pdf} missing_media={missing_media} failed={failed}"
    );
    Ok(())
}

enum MediaIdLookup {
    Found(String),
    /// `mediaCloudFileId` is explicitly null — document has no backing PDF.
    Null,
    Malformed,
}

/// Inspect the PDF metadata blob from `documents.content` and report the state
/// of `payload.mediaCloudFileId`.
fn media_cloud_file_id_from_metadata(content: &[u8]) -> MediaIdLookup {
    let Ok(v) = serde_json::from_slice::<Value>(content) else {
        return MediaIdLookup::Malformed;
    };
    let Some(field) = v.get("payload").and_then(|p| p.get("mediaCloudFileId")) else {
        return MediaIdLookup::Malformed;
    };
    if field.is_null() {
        return MediaIdLookup::Null;
    }
    match field.as_str() {
        Some(s) if !s.is_empty() => MediaIdLookup::Found(s.to_string()),
        _ => MediaIdLookup::Malformed,
    }
}

/// Try to extract text from a PDF. Both pdf-extract versions panic on
/// different malformed inputs, so we try 0.10 first and fall back to 0.7;
/// each call is wrapped in `catch_unwind`.
///
/// Empirically these two versions disagree on ~36 of our 408 PDFs: 0.10
/// handles function-type-4 PDFs that 0.7 panics on, while 0.7 handles 16
/// PDFs with non-UTF-8 CMap streams that 0.10 panics on. Only 2 PDFs in the
/// corpus defeat both.
pub fn extract_pdf_text(bytes: &[u8]) -> Option<String> {
    if let Some(text) = try_with(bytes, |b| pdf_extract::extract_text_from_mem(b).ok()) {
        return Some(text);
    }
    try_with(bytes, |b| pdf_extract_old::extract_text_from_mem(b).ok())
}

fn try_with<F>(bytes: &[u8], extract: F) -> Option<String>
where
    F: FnOnce(&[u8]) -> Option<String> + panic::UnwindSafe,
{
    let bytes = bytes.to_vec();
    let result = with_silenced_stdio(|| {
        panic::catch_unwind(panic::AssertUnwindSafe(move || extract(&bytes)))
    });
    match result {
        Ok(Some(text)) if !text.trim().is_empty() => Some(text),
        _ => None,
    }
}

/// pdf-extract 0.7 prints diagnostics via raw `println!`/`eprintln!`, and 0.10
/// uses `log::warn!` which may still leak through if a logger is installed.
/// Redirect fd 1 and fd 2 to /dev/null for the duration of the call, then
/// restore them. Best-effort: if any libc call fails we just run unsilenced.
fn with_silenced_stdio<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let guard = StdioSilencer::new();
    let result = f();
    drop(guard);
    result
}

struct StdioSilencer {
    saved_stdout: Option<c_int>,
    saved_stderr: Option<c_int>,
    devnull: Option<c_int>,
}

impl StdioSilencer {
    fn new() -> Self {
        let devnull = CString::new("/dev/null").unwrap();
        unsafe {
            let null_fd = libc::open(devnull.as_ptr(), libc::O_WRONLY);
            if null_fd < 0 {
                return Self { saved_stdout: None, saved_stderr: None, devnull: None };
            }
            // Make sure Rust's line-buffered stdout flushes before redirecting.
            libc::fflush(std::ptr::null_mut());
            let saved_stdout = dup_fd(libc::STDOUT_FILENO);
            let saved_stderr = dup_fd(libc::STDERR_FILENO);
            if saved_stdout.is_some() {
                libc::dup2(null_fd, libc::STDOUT_FILENO);
            }
            if saved_stderr.is_some() {
                libc::dup2(null_fd, libc::STDERR_FILENO);
            }
            Self { saved_stdout, saved_stderr, devnull: Some(null_fd) }
        }
    }
}

impl Drop for StdioSilencer {
    fn drop(&mut self) {
        unsafe {
            libc::fflush(std::ptr::null_mut());
            if let Some(fd) = self.saved_stdout {
                libc::dup2(fd, libc::STDOUT_FILENO);
                libc::close(fd);
            }
            if let Some(fd) = self.saved_stderr {
                libc::dup2(fd, libc::STDERR_FILENO);
                libc::close(fd);
            }
            if let Some(fd) = self.devnull {
                libc::close(fd);
            }
        }
    }
}

unsafe fn dup_fd(fd: c_int) -> Option<c_int> {
    let copy = libc::dup(fd);
    if copy < 0 { None } else { Some(copy) }
}

fn normalize_whitespace(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last_was_space = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            if !last_was_space {
                out.push(' ');
                last_was_space = true;
            }
        } else {
            out.push(ch);
            last_was_space = false;
        }
    }
    out.trim().to_string()
}
