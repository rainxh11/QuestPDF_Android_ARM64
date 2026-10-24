# QuestPDF Rust port

This workspace reimplements QuestPDF's managed composition, layout, pagination,
text, drawing, semantic structure, and document operations in Rust. Rendering uses
the original native Skia ABI; PDF operations use the original native qpdf ABI.
The Rust library and RIS applications do not invoke .NET.

The baseline is **QuestPDF 2026.9.1**, source commit
`bd5b6609c09a8db8bbea95041a9851c8811ea5ce`, Skia ABI **26**, qpdf ABI **4**.
The existing .NET implementation is retained. Both supplied RIS C# projects now
pin the same **2026.9.1** package, replacing their previous 2026.7.1 reference.

The port targets close API and output compatibility. Rust ownership, closures,
traits, snake_case names, and `Result` replace CLR-specific conventions. See
[PORT_STATUS.md](PORT_STATUS.md) for precise coverage, evidence, and differences.
[VERIFICATION.md](VERIFICATION.md) records the completed comparison runs.

## Projects

| Crate | Responsibility |
| --- | --- |
| `questpdf` | Fluent API, stateful Measure/Draw engine, fonts, native rendering, semantic tags, Companion v3, PDF operations |
| `ris-document-server` | RIS models, six document types / seven variants, matching C ABI and receipt printing |
| `receipt-preview` | Seven sample documents, Companion preview, PDF/image generation and optional printing |

New code is isolated in this directory. Embedded Lato fonts, MIME data, native
libraries and RIS assets retain their upstream licensing. License texts and
attribution accompany release bundles.

## Build and run

Run from this directory:

```powershell
cargo build --workspace --locked
cargo run -p questpdf --example hello -- output/hello.pdf
cargo run -p receipt-preview
cargo run -p receipt-preview -- --report large-receipt --preview
cargo run -p receipt-preview -- --images --escpos
```

The CLI writes all seven PDFs to `output/rust` by default. `--report` selects the
preview/print document; `--debt` selects the debt receipt. Options and report names
are case insensitive. `--fixtures` and `--output` override directories.

`--preview` uses QuestPDF Companion API v3, as the C# program does. The installed
`questpdf-companion` executable is launched if necessary; `--companion-port`
changes the default 12500. `--viewer` instead opens the saved PDF in the OS viewer.
`--print --printer HOST --printer-port PORT` sends a network print job.
`--escpos` only saves local byte streams.

Native dependencies resolve in this order: explicit `QUESTPDF_SKIA_PATH` /
`QUESTPDF_QPDF_PATH`, beside the Rust executable or loaded shared library, its
`runtimes/<rid>/native` directory, then the development checkout's existing .NET
runtime directory. `DOC_REPORTS_ASSETS_PATH` overrides RIS assets; release bundles
otherwise use `Assets` beside the Rust library. Matching native ABI versions are
checked before use.

## Similar API

```rust
use questpdf::*;

Settings::set_license(LicenseType::Evaluation); // Declare the applicable tier.
let document = Document::create(|container| {
    container.page(|page| {
        page.size(PageSizes::A4);
        page.margin_unit(2.0, Unit::Centimetre);
        page.default_text_style(|s| s.font_size(20.0));
        page.header().text("Hello PDF!").semi_bold().font_size(36.0);
        page.content().column(|column| {
            column.spacing(12.0);
            column.item().text("Measured and paginated in Rust.");
        });
        page.footer().align_center().text_spans(|text| {
            text.span("Page ");
            text.current_page_number();
            text.span(" of ");
            text.total_pages();
        });
    });
});
document.generate_pdf("report.pdf")?;
# Ok::<(), anyhow::Error>(())
```

`IDocument` and `IComponent` provide reusable document/component composition.
`Document::merge` supports original or continuous page numbers, with scoped
sections. Flexible page dimensions, dynamic components, lazy content, native
image/SVG exports, font management, colors, page sizes and font feature catalogs
are included. Companion sessions support explicit document updates and polling.

Configure global settings before first use:

```rust
Settings::configure(|s| {
    s.license = Some(LicenseType::Evaluation);
    s.use_system_fonts = false;
    s.font_discovery_path = Some("Fonts".into());
    s.enable_caching = true;
});
FontManager::register_font_from_file("Fonts/Custom.ttf")?;
```

All 18 Lato faces are embedded. Automatic font discovery scans the application
directory recursively once, unless disabled or redirected. System fonts are
opt-in, matching the 2026.9.1 default. A `Renderer` can also own registrations.
The RIS applications preserve their original Community declaration.

`Document::create` constructs an owned layout tree immediately. Use `IDocument`
when composition must be repeated from changing data. Build documents per render
thread; native handles and the layout tree have explicit Rust ownership.

## Verification

```powershell
python verification/verify.py
python verification/verify_companion.py
```

The full driver builds/tests Rust, generates independent C# references from both
the updated NuGet package and this source checkout, and writes every log and
comparison into a fresh output directory. `--source` changes the RIS source root.
`--pdf-only` omits the separate native PNG/ESC-POS comparison.

Checks cover seven RIS variants / 16 pages, 163 engine cases (including three
expected failures), four merged-document cases, 21 JSON DateTime Kind/offset
cases, global settings, native encoders, font enumeration, C ABI buffers/errors,
loopback TCP printing, and qpdf operations including 40/128/256-bit encryption.

PDF comparisons require identical page counts, sizes, text and links, text bounds
within 0.001 point, and **exact RGB pixels at 144 DPI**. RIS native PNG exports at
**150 DPI** and three thermal ESC/POS streams match exactly. PDF file bytes are
not asserted identical: metadata and object serialization can differ.

Python needs `pymupdf`, `numpy`, `pillow`, and `pypdf`. C# oracles need .NET 10.
Companion checks need Postman CLI >=1.70. They run real C# and Rust clients against
local Postman mocks derived from the checked-in C# v3 contract, compare JPEG bytes
at two zoom levels and element bounds, and check error/refresh/cancellation paths.
No external mock is deployed. The owner's published mock lookup was unavailable
while signed out, so the source contract is the independent reference.

If `verification/artifacts/verapdf-tools/runtime-1.30.3/bin/cli-1.30.3.jar` exists,
the full driver also runs veraPDF. Supply `--verapdf-jar PATH` otherwise. Formal
validation covers PDF/A-2a/2b/2u, PDF/A-3a/3b/3u, PDF/UA-1, combined A+UA, and tagged
merged documents. Missing optional tools are reported explicitly as skipped.

Linux x64 verification uses a pinned official Rust Docker image and the existing
Linux native libraries. Pass the reference directories from a Windows full run:

```powershell
python verification/verify_linux.py --reference output/verification-RUN/project --engine-reference output/verification-RUN/engine-dotnet
```

It builds/runs Rust without a .NET runtime in the container and compares RIS PDFs,
images, print streams, the C ABI, PDF operations and engine cases. The Windows
Arial system-font fixture is excluded explicitly. Timezone must match the C#
baseline: the driver uses POSIX `CET-1`, equivalent to Africa/Algiers for these dates.

## Release bundles

```powershell
python package_release.py
python package_release.py --rid linux-x64 --binaries target/linux/release --skip-build
python verification/verify_package.py output/RELEASE-DIRECTORY --reference output/verification-RUN/project
```

Each creates a new directory and ZIP under `output`, including the CLI, RIS shared
library, native Skia/qpdf, assets, fixtures, C header, license texts and a SHA-256
manifest. Existing output directories are never replaced. Linux ZIPs preserve
executable permissions. Rust application runs require no .NET installation.
The bundle verifier disables development runtime lookup, checks file hashes, and
compares CLI/C ABI PDFs, native images, and print streams with the C# reference.

The C contract is [ris_document_reports.h](document-server/include/ris_document_reports.h).
Release both PDF and error buffers with `doc_reports_free_buffer`.
Builds, generated outputs, oracle copies, and test captures are ignored by Git.
