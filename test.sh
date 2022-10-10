#!/bin/bash
set -eux
cargo build -r
target/release/tzgrep --help
pat="hogeP"
time target/release/tzgrep "$pat" test_data.tar
time target/release/tzgrep "$pat" test_data.tar.gz
time target/release/tzgrep "$pat" test_data.tar.bz2
time target/release/tzgrep "$pat" test_data.tar.xz
time target/release/tzgrep "$pat" test_data.tar.zst
time target/release/tzgrep "$pat" test_data.tar.zst
time target/release/tzgrep "$pat" test_data2.tar.zst
time target/release/tzgrep "$pat" test_data3.tar.zst
time target/release/tzgrep "$pat" test_data4.tar.zst
time target/release/tzgrep "$pat" test_data5.tar.zst

set +e
for i in $(seq 5); do
    time find test_data/ -type f | xargs grep hogeP
    time rg hogeP test_data/
    time tar xf test_data.tar -O | rg hogeP
    time target/release/tzgrep "$pat" test_data.tar
done
time tar xf test_data.tar --to-command='grep --label=$TAR_FILENAME -H hogeP; true'
