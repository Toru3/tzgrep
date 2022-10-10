#!/bin/sh
set -eu
mkdir -p test_data/
cd test_data/
seq 1000000 | xargs -P"$(nproc)" -I{} sh -c 'echo {}.txt; head -c 512 /dev/urandom 2>/dev/null | base64 >{}.txt'
cd ..
tar cvf test_data.tar test_data/
for c in gzip bzip2 xz zst; do
    "$c" -k test_data.tar &
done
tar zcf test_data2.tar.gz -H oldgnu test_data/ &
tar zcf test_data3.tar.gz -H pax    test_data/ &
tar zcf test_data4.tar.gz -H ustar  test_data/ &
tar zcf test_data5.tar.gz -H v7     test_data/ &
wait
