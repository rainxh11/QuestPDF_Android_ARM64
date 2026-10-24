#!/bin/sh
set -eu
cd /workspace
output=${1:-output/linux}
rustc --version
cargo build --workspace --locked --release
cargo run --release --locked -p receipt-preview -- --output "$output"/rust --images --escpos
cargo run --release --locked -p questpdf --example pdf_operations -- "$output"/operations
cargo run --release --locked -p questpdf --example merged_documents -- "$output"/merged-rust
cc -Wall -Werror verification/ffi_smoke.c -I document-server/include -L target/linux/release -lris_document_reports '-Wl,-rpath,/workspace/target/linux/release' -o "$output"/ffi-smoke
mkdir -p "$output"/ffi
for fixture in receipt:receipt debt-receipt:debt-receipt large-receipt:large-receipt debt-report:debt debt-out-report:debt-out income-report:income substitute-report:substitute
do
    name=${fixture%:*}
    kind=${fixture#*:}
    "$output"/ffi-smoke "verification/fixtures/$name.json" "$kind" "$output/ffi/$name.pdf"
done
if test -f "$output"/engine-cases.json
then
    cargo run --release --locked -p questpdf --example engine_cases -- "$output"/engine-cases.json "$output"/engine-rust
fi
