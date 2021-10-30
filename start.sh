#!/bin/bash
cargo run -- --host 15.161.71.249 --port 80 --path staging/api/v1 --network skynet
#cargo run --bin trinci-cli -- \
#    --host t2.dev.trinci.net \
#    --host 10.0.0.65 \
#    --host 127.0.0.1 \
#    --port 80 \
#    --path api/v1 \
#    --network nightly \
    "$@"


    #http://t2.dev.trinci.net/nightly