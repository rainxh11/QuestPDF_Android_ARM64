# Compatibility and verification status

Baseline: QuestPDF **2026.9.1**, commit
`bd5b6609c09a8db8bbea95041a9851c8811ea5ce`; Skia ABI 26 / qpdf ABI 4.
Rust composition, measurement, drawing and pagination execute without a CLR.
The .NET implementation remains in its original folder.

## Implemented core

| Source area | Rust counterpart |
| --- | --- |
| Infrastructure / Minimal API | `infrastructure.rs`, `fluent.rs`, `settings.rs`: document/component traits, metadata/settings, units, sizes/colors/styles, page descriptors, merged documents |
| Stateful Elements | `elements.rs`: rows, columns, inlined/grid/multicolumn layouts, constraints, padding/alignment, tables with placement/spans/repeated headers/footers/extended last cells, decoration/layers |
| Flow and transforms | Pagination, breaks, ensure space, prevent breaks, show/skip/repeat/conditional content, scale/flips/rotation, shrink/extend/unconstrained content, sections/links, ZIndex |
| Text | Native paragraph shaping, fallback families, OpenType features, weights/styles/decorations, line/paragraph spacing, indentation/clamping, injected elements, page/section numbers and formatters |
| Images / styling | Raster/SVG/dynamic media, original/compressed/downsampled images, per-edge borders, corner radii, gradients and shadows, placeholders and SVG paths |
| Drawing | Two-pass layout, native picture layers, snapshot/measurement caching, content-position capture, PDF/PNG/JPEG/WebP/SVG exports |
| Dynamic / Lazy | Cloneable dynamic state with measurement rollback, dynamic elements, lazy content and lazy caching |
| Semantics | Logical structure trees, artifacts, headings/lists/links/figures, table headers/spans, language/alternate descriptions, PDF/A and PDF/UA metadata |
| Companion | API v3 handshake, heartbeat, native page snapshots, JPEG rendering by zoom, Rust element hierarchy/bounds, source pointers, error notifications, explicit refresh and cancellation |
| Font management | Embedded Lato faces, file/stream/directory registration, automatic discovery, registered/system font enumeration and opt-in system fallback |
| PDF operations | Selection/order, merging, layers, attachments, metadata extension, linearization, encryption/decryption and restrictions |
| RIS applications | Original models and six template types / seven variants, C ABI allocation/free/status codes, printer rasterization/ESC-POS and TCP transport, preview CLI |

## Evidence

- Both supplied C# csproj files were updated from 2026.7.1 to **2026.9.1**.
  Those are the only intentional edits in the external RIS repository.
- All seven RIS PDFs match the updated package and direct-source baselines: receipt,
  debt receipt, large receipt, debt report (4 pages), debt-out report, income report
  (6 pages), substitute report (2 pages): **16 pages**.
- PDF comparison checks exact 144-DPI RGB pixels, text/links/page counts/sizes and
  text bounds within 0.001 point. Native RIS PNG exports at 150 DPI and all three
  ESC/POS byte streams also match. No physical printer job was sent.
- **163 engine cases**, four merged-document cases / 20 pages, five settings PDFs,
  and **21 JSON timestamp variants / 48 pages** have independent C# counterparts.
  Three engine errors are explicit expected failures; unexpected matching errors
  are not counted as successful PDFs.
- Styled native export fixture is byte-identical for opaque/transparent PNG, JPEG,
  WebP and SVG. Font metadata enumeration and precise decimal formatting match.
- Actual C ABI checks cover all seven generators, invalid/missing input, allocator
  release, null collections and case-insensitive fields. TCP tests capture only
  localhost and require exact reference print bytes.
- veraPDF validates the archival/accessibility corpus against PDF/A-2a/2b/2u,
  PDF/A-3a/3b/3u and PDF/UA-1, including combined A+UA and merged documents.
  Logical tree comparisons additionally check tags, attributes, bounds and IDs.
- Companion's real C# and Rust clients are tested through local Postman servers.
  JPEG frame bytes and text bounds match; version rejection, invalid page indices,
  layout errors, refresh, heartbeats and cancellation are exercised. This is
  protocol verification; the desktop Companion window was not operated.
- Linux x64 builds/runs use native ABI-compatible Linux libraries with no .NET
  installation in the container. Results and platform boundaries are recorded
  in the generated verification directories.

## Deliberate adaptations and limits

- Rust uses ownership, traits, closures, snake_case names, and `Result` in place of
  CLR conventions. `Document::create` composes immediately into an owned tree;
  use `IDocument` for recomposition from changing data. Layout instances are not
  shared between render threads.
- Companion hierarchy describes Rust elements/source locations, so its full tree
  is not identical to the C# generated wrapper hierarchy. Rendered frame bytes and
  tested content bounds match. Automatic .NET MetadataUpdater hot reload and its
  async/thread-pool integration have no direct Rust equivalent; use explicit
  `CompanionSession::update_document`, `poll`, and cancellable `run`.
- Removed C# XPS generation and deprecated Canvas callbacks are omitted rather than
  exposed as working Rust features. Their C# methods already fail or are compile
  errors in 2026.9.1. Historical aliases generally map to the modern equivalents.
- Saving PDF operations over their own input path is rejected to avoid truncation.
  Attachment dates use valid two-digit seconds; source C# uses `HHmmsss`, which
  qpdf rejects. These are deliberate functional corrections.
- Source Lazy + balanced MultiColumn observer behavior is retained; its regression
  fixture drops the same final item as C#. Source gradient angle 270 behavior and
  page-level direction overriding the global default are also retained.
- Verified runtime platforms are Windows x64 and Linux x64. Rust execution on
  Android/ARM/macOS is not claimed. Existing .NET Android support is preserved.
- Output equality is rendering/content equality, not PDF-file byte identity or
  proof of every possible document combination. Native dependencies and fonts
  must match; system-font fixtures depend on the host's installed fonts.

## Reproduce

`python verification/verify.py` creates a fresh directory and fails for any build,
expected-file, layout, content, pixel, semantic or conformance mismatch. Logs and
quantitative JSON reports are retained. `verify_companion.py`, `verify_json_cases.py`
and `verify_linux.py` provide focused drivers. `package_release.py` produces new
standalone runtime bundles with file hashes. See README for prerequisites.
