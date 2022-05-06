#!/bin/sh

cd ../..
mkdir cmake-build
cmake . -Bcmake-build
bindgen cmake-build/src/mongocrypt.h \
    -o bindings/rust/src/bindings.rs \
    --allowlist-function 'mongocrypt_.*' \
    -- -I src
