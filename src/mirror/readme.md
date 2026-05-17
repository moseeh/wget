# `mirror` module

Recursive website mirroring for `--mirror`.

## Files

- `crawler.rs` — `MirrorCrawler`, the BFS engine. Owns the visited set, queue, and the URL → local-path map used by `--convert-links`.
- `parser.rs` — link extraction, reject/exclude filters, and the HTML/CSS link rewriters.
- `mod.rs` — re-exports.

## Crawl flow

1. Seed the queue with the starting URL.
2. Pop a URL, skip if visited / rejected by `-R` / excluded by `-X`.
3. Fetch via `HttpClient::download_silent`. The response is consumed as raw bytes (`response.bytes()`) so binary files survive intact.
4. Classify the response as `Html`, `Css`, or `Other` — first by `Content-Type` header, then by a sniff of the first 512 bytes (`<!doctype`/`<html`). The classification is recorded so the rewrite pass can target HTML/CSS only.
5. Write the body to a local path derived from the URL (`<output>/<host>/<path>`; bare directories get `index.html`).
6. If the asset is HTML, extract links and enqueue.
7. After the queue drains, if `--convert-links` is set, rewrite saved HTML/CSS to point at local files.

## Link extraction

Per spec, only three tag/attribute pairs are followed:

- `<a href="...">`
- `<link href="...">`
- `<img src="...">`

Other `href`/`src` attributes (`<script src>`, `<iframe src>`, …) are ignored. In-page anchors (`#…`) and `mailto:`/`javascript:`/`data:`/`tel:` URIs are skipped. Cross-domain links are skipped — only same-`domain` + same-`scheme` URLs are queued.

## `-R` reject

Reject is keyed on the path component's file extension after URL parsing, so `/img.jpg?v=2` is correctly identified as `.jpg`. Suffixes are case-insensitive and leading dots are tolerated (`-R=.jpg,gif`).

## `-X` exclude

Excludes path prefixes. `-X=/img,/cache` skips any URL whose path starts with `/img` or `/cache` (as a complete segment).

## `--convert-links`

After the crawl, every saved HTML and CSS file is read back and rewritten:

- HTML: `href`/`src` on `<a>` / `<link>` / `<img>` whose absolute URL appears in the saved-URL map is rewritten to a path relative to the document's directory.
- CSS: `url(...)` references receive the same treatment.

Unresolved targets (external sites, schemes we don't handle, `data:` URIs, fragments) are left untouched.
