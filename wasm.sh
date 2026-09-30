#!/bin/sh
set -e
wasm-pack build -t web --dev -d ./site/pkg
npx http-server ./site -p 8000

