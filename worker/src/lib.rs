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
    if req.method() != Method::Get {
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
            headers.set("Cache-Control", "public, max-age=600")?;
        } else {
            headers.set("Cache-Control", "public, max-age=31536000, immutable")?;
        }
        headers.set("X-Omapak-Origin", if is_summary { "omapak" } else { "cached" })?;
        let body = match obj.body() {
            Ok(b) => b,
            Err(e) => return Response::error(format!("object body: {e}"), 500),
        };
        let _ = body;
        return obj.as_response().with_headers(headers).into();
    }

    // Cache miss: fetch from flathub once, store, serve. Content-addressed
    // paths make this safe forever; upstream 404s pass through as 404s.
    let mut init = RequestInit::new();
    init.method = Method::Get;
    let upstream_req = Request::new_with_init(&format!("{UPSTREAM}/{path}"), &init)?;
    match Fetch::Request(upstream_req).send().await {
        Ok(resp) if resp.status_code() == 200 => {
            let bytes = resp.bytes().await?;
            let _ = bucket.put(&path, bytes.clone()).execute().await;
            let mut headers = Headers::new();
            headers.set("Content-Type", "application/octet-stream")?;
            headers.set("Cache-Control", "public, max-age=31536000, immutable")?;
            headers.set("X-Omapak-Origin", "flathub-pass-through")?;
            Response::from_bytes(bytes)
                .map(|r| r.with_headers(headers))
                .into()
        }
        _ => Response::error("not found", 404).into(),
    }
}
