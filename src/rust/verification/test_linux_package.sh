#!/bin/sh
set -eu
output=$1
/package/receipt-preview --output "$output/cli" --images --escpos
cc -Wall -Werror /workspace/verification/ffi_smoke.c -I /package -L /package -lris_document_reports '-Wl,-rpath,/package' -o "$output/ffi-smoke"
mkdir -p "$output/ffi"
for fixture in receipt:receipt debt-receipt:debt-receipt large-receipt:large-receipt debt-report:debt debt-out-report:debt-out income-report:income substitute-report:substitute
do
    name=${fixture%:*}
    kind=${fixture#*:}
    "$output/ffi-smoke" "/package/fixtures/$name.json" "$kind" "$output/ffi/$name.pdf"
done
