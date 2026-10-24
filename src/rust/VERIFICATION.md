# Verified port results — 2026-10-24

QuestPDF baseline **2026.9.1**, source commit
`bd5b6609c09a8db8bbea95041a9851c8811ea5ce`, native Skia ABI **26**, qpdf ABI **4**.
Both supplied RIS C# projects now reference that exact package version.
The original .NET library and applications remain available.

## Outcome

All seven Rust RIS documents match independent C# outputs from both the updated
NuGet package and this checkout. Every page has identical rendered RGB pixels at
144 DPI, text, links and dimensions; text positions agree within 0.001 point.
This verifies content/rendering equality; PDF serialization bytes can differ.

| Document | Pages | Changed rendered pixels |
| --- | ---: | ---: |
| Receipt | 1 | 0 |
| Debt receipt | 1 | 0 |
| Large receipt | 1 | 0 |
| Debt report | 4 | 0 |
| Debt-out report | 1 | 0 |
| Income report | 6 | 0 |
| Substitute report | 2 | 0 |

All sixteen native RIS PNG pages at 150 DPI and all three receipt ESC/POS byte
streams also match exactly. C ABI generation and buffer/error contracts pass;
printing was captured over localhost TCP without sending a physical printer job.

## Additional verification

- 163 independent engine cases: 160 successful documents and three explicitly
  expected invalid-input/layout errors.
- Four merged documents / 20 pages, including continuous/original numbering,
  sections, links and PDF/UA tags.
- 21 JSON timestamp variants / 48 pages, covering UTC, numeric offsets and
  unspecified DateTime kinds across all seven RIS variants.
- Five settings PDFs / six pages, including license metadata, font discovery,
  caching, unknown font fallback and application-relative media resources.
- PNG, transparent PNG, JPEG, WebP and SVG native export bytes; registered and
  system font metadata; exact decimal formatting.
- Independent qpdf checks for selection, merge, layering, attachments, metadata,
  linearization and 40/128/256-bit encryption, permissions and decryption.
- Sixteen tagged/archive documents, seventeen veraPDF validation profiles per
  implementation: zero failed rules for the tested PDF/A and PDF/UA corpus.
- Companion v3: real C# and Rust clients against local Postman mock/simulation,
  exact JPEG frames at two zoom levels, content bounds, invalid indices,
  handshake rejection, errors, refresh, heartbeats and cancellation. The actual
  Rust receipt-preview CLI lifecycle also passes. No desktop Companion GUI was
  operated.
- Linux x64: all seven RIS variants, native image/print exports, actual C ABI,
  PDF operations, merged samples and 162 engine cases pass in a Rust container
  without .NET. The Windows Arial system-font case is explicitly excluded.
- Windows and Linux standalone bundles pass with development native-library
  lookup disabled and assets/native libraries resolved from the bundle.
- `cargo fmt --check`, Clippy with warnings denied, workspace tests/builds, and
  independent `cargo package -p questpdf --allow-dirty` verification pass.

## Retained local evidence

Generated directories are ignored by Git; the drivers recreate them without
deleting previous runs. Paths below are relative to this Rust workspace.

| Evidence | Location |
| --- | --- |
| Full Windows run, step exit codes and logs | `output/verification-20261024-162027-198e76/steps.json` |
| RIS package comparison | `output/verification-20261024-162027-198e76/compare-ris-package/results.json` |
| RIS checkout comparison | `output/verification-20261024-162027-198e76/compare-ris-checkout/results.json` |
| Native image/print comparison | `output/verification-20261024-162027-198e76/image-comparison.json` |
| Latest five-settings comparison | `output/comparison-settings/results.json` |
| Latest JSON case run | `output/json-verification-20261024-160415-515462` |
| Latest focused Companion run | `output/companion-verification-20261024-161617-363701` |
| Latest full Linux run | `output/linux-verification-20261024-161026-155930` |
| Windows bundle proof | `output/package-verification-20261024-163014-001965` |
| Linux bundle proof | `output/package-verification-20261024-163014-003136` |

Both bundle proofs compare the release binaries against the full Windows run's
C# package output, with exact native PNG/ESC/POS and PDF-content comparisons.
Release directories and ZIPs include `manifest.json` with SHA-256 file hashes.

See [README.md](README.md) for commands and prerequisites, and
[PORT_STATUS.md](PORT_STATUS.md) for deliberate Rust/CLR adaptations, source quirks
and platform limitations. These results cover the checked corpus, not every
possible document combination. Rust Android/ARM/macOS execution is unverified.
