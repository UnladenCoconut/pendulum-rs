#!/bin/sh
set -e
wasm-pack build -t web --dev
npx http-server ./ -p 8000

