use std::path::Path;
use walkdir::WalkDir;

const TOTAL_BUDGET_BYTES: usize = 350_000;
const PER_FILE_CAP_BYTES: usize = 24_000;

const TEXT_EXTENSIONS: &[&str] = &[
    "rs", "c", "h", "cpp", "cc", "py", "js", "ts", "jsx", "tsx", "svelte", "vue", "go", "java",
    "kt", "sh", "vala", "ui", "xml", "yml", "yaml", "json", "toml", "md", "txt", "css", "html",
    "meson", "build", "cmake", "glsl", "desktop", "metainfo.xml", "appdata.xml", "gresource",
];

/// Compact, budgeted text digest of a source tree for the judge prompt: the
/// tree with sizes, then the contents of small text files. Binaries are
/// listed but never inlined — the model sees names and sizes only.
pub fn build(source_dir: &Path) -> String {
    let mut listing = String::new();
    let mut contents = String::new();
    let mut remaining = TOTAL_BUDGET_BYTES;
    let mut file_count = 0;

    let mut entries: Vec<_> = WalkDir::new(source_dir)
        .into_iter()
        .filter_entry(|e| {
            let n = e.file_name().to_string_lossy();
            n != ".git" && n != "node_modules" && n != "target" && n != "build"
        })
        .flatten()
        .filter(|e| e.file_type().is_file())
        .collect();
    entries.sort_by_key(|e| e.path().to_path_buf());

    for entry in entries {
        let Ok(meta) = entry.metadata() else { continue };
        let rel = entry
            .path()
            .strip_prefix(source_dir)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .into_owned();
        file_count += 1;
        listing.push_str(&format!("{:>9}  {}\n", meta.len(), rel));

        if remaining == 0 || meta.len() as usize > PER_FILE_CAP_BYTES {
            continue;
        }
        let ext = entry
            .path()
            .extension()
            .map(|e| e.to_string_lossy().into_owned())
            .unwrap_or_default()
            .to_lowercase();
        if !TEXT_EXTENSIONS.contains(&ext.as_str()) {
            continue;
        }
        let Ok(bytes) = std::fs::read(entry.path()) else { continue };
        if bytes.contains(&0) {
            continue; // extension lied; it's binary
        }
        let len = bytes.len().min(remaining);
        let text = String::from_utf8_lossy(&bytes[..len]);
        contents.push_str(&format!("\n----- {rel} -----\n{text}\n"));
        remaining -= len;
    }

    let _ = file_count;
    format!("TREE ({file_count} files):\n{listing}\nFILE CONTENTS (budget {}/{} bytes used):\n{contents}",
        TOTAL_BUDGET_BYTES - remaining, TOTAL_BUDGET_BYTES)
}
