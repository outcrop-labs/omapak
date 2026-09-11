//! omapak repo proxy: serves repo.omapak.org from the omapak-repo R2 bucket,
//! falling back to dl.flathub.org for anything missing and caching what it
//! fetches. This is what makes a single omapak remote cover the entire
//! flathub catalog: one remote-add, every app, updates included.
//!
//! Freshness: OSTree objects are content-addressed and immutable, so a
//! cached object can never be stale; new releases are new object paths,
//! which miss the cache and fetch from upstream. The summary (generated
//! daily by CI, signed with omapak's key) is the only freshness surface.

use worker::*;

const UPSTREAM: &str = "https://dl.flathub.org/repo";

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    if req.method() != Method::Get && req.method() != Method::Head {
        return Response::error("method not allowed", 405);
    }

    let bucket = env.bucket("OMAPAK_REPO")?;
    let path = req.path().trim_start_matches('/').to_string();
    if path.is_empty() {
        return Response::error("not found", 404);
    }



    // R2 first (omapak's own objects, plus everything previously cached).
    if let Some(obj) = bucket.get(&path).execute().await? {
        let is_summary = path.starts_with("summary");
        let mut headers = Headers::new();
        headers.set("Content-Type", "application/octet-stream")?;
        if is_summary {
            headers.set("Cache-Control", "public, max-age=60, must-revalidate")?;
        } else {
            headers.set("Cache-Control", "public, max-age=31536000, immutable")?;
        }
        headers.set("X-Omapak-Origin", if is_summary { "omapak" } else { "cached" })?;
        // worker 0.8: body() is Option<ObjectBody>; response_body() hands
        // the stream to the runtime without buffering it in the worker.
        let _ = headers.set("Access-Control-Allow-Origin", "*");
        let body = obj.body().ok_or_else(|| Error::RustError("object body consumed".into()))?;
        return Ok(Response::from_body(body.response_body()?)?.with_headers(headers));
    }

    // Repo-critical files must come from R2 only. A flathub fallback 200
    // here would cache upstream bytes as omapak's own (a flathub summary
    // masquerading as ours, an upstream .flatpakrepo re-keying clients),
    // which no omapak keyring could ever verify.
    let repo_critical = path.starts_with("summary")
        || path == "config"
        || path == "omapak.flatpakrepo";
    if repo_critical {
        return Response::error("not found", 404);
    }

    // Cache miss: fetch from flathub once, store, serve. Content-addressed
    // paths make this safe forever. Anything but a clean upstream 200
    // becomes a 404: clients (rightly) treat speculative paths like
    // deltas/ as miss-and-fallback, and a thrown 500 would abort installs.
    async fn fetch_upstream(path: &str, bucket: &worker::Bucket) -> Result<Response> {
        let mut init = RequestInit::new();
        init.method = Method::Get;
        let upstream_req = Request::new_with_init(&format!("{UPSTREAM}/{path}"), &init)?;
        let mut resp = Fetch::Request(upstream_req).send().await?;
        if resp.status_code() != 200 {
            return Response::error("not found", 404);
        }

        let mut headers = Headers::new();
        headers.set("Content-Type", "application/octet-stream")?;
        headers.set("Cache-Control", "public, max-age=31536000, immutable")?;

        // Deltas run to hundreds of MB and would blow the worker's memory
        // if buffered; stream them through untouched. Same for any object
        // claiming more than 64MB. Everything bounded gets cached.
        let content_length: Option<usize> = resp
            .headers()
            .get("Content-Length")
            .ok()
            .flatten()
            .and_then(|v| v.parse().ok());
        let huge = path.starts_with("deltas/") || content_length.is_some_and(|n| n > 8 * 1024 * 1024);
        if huge {
            let _ = headers.set("Access-Control-Allow-Origin", "*");
            headers.set("X-Omapak-Origin", "flathub-stream")?;
            return Ok(resp.with_headers(headers));
        }

        let bytes = resp.bytes().await?;
        let _ = bucket.put(path, bytes.clone()).execute().await;
        let _ = headers.set("Access-Control-Allow-Origin", "*");
        headers.set("X-Omapak-Origin", "flathub-pass-through")?;
        Ok(Response::from_bytes(bytes)?.with_headers(headers))
    }

    match fetch_upstream(&path, &bucket).await {
        Ok(resp) => Ok(resp),
        Err(e) => {
            console_debug!("upstream fetch failed for {path}: {e:#}");
            Response::error("not found", 404)
        }
    }
}
