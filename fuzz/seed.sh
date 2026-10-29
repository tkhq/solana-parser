#!/usr/bin/env sh
# Seed fuzz corpora from data the library already embeds or tests against, so
# the seeds track the source instead of living as committed copies. Idempotent.
set -eu
cd "$(dirname "$0")"

# fuzz_idl_json: the IDLs the library embeds.
mkdir -p corpus/fuzz_idl_json
cp ../src/solana/idls/*.json corpus/fuzz_idl_json/

# fuzz_transaction: every hex transaction literal in src/solana/tests.rs,
# behind each of the target's mode bytes. Modes 00/01 carry the decoded bytes
# (the target hex-encodes them); 02/03 carry the hex text as-is (the target
# passes it through unchanged). Bit 0 is `full_transaction`.
mkdir -p corpus/fuzz_transaction
grep -ohE '"[0-9a-fA-F]{64,}"' ../src/solana/tests.rs | tr -d '"' | sort -u |
    while read -r tx; do
        sum=$(printf '%s' "$tx" | cksum | cut -d' ' -f1)
        for mode in 00 01; do
            printf '%s%s' "$mode" "$tx" | perl -ne 'print pack("H*", $_)' \
                >"corpus/fuzz_transaction/tests-$mode-$sum"
        done
        for mode in 02 03; do
            { printf '%s' "$mode" | perl -ne 'print pack("H*", $_)'; printf '%s' "$tx"; } \
                >"corpus/fuzz_transaction/tests-$mode-$sum"
        done
    done
