#!/bin/sh
#use for slang-rs / shader-slang crate
export SLANG_INCLUDE_DIR=/usr/include/shader-slang/
export SLANG_DIR=/usr/lib
export BINDGEN_EXTRA_CLANG_ARGS="-Wno-register"
