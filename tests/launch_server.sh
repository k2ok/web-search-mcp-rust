#!/bin/sh

cd `dirname $0`

pwd
../target/release/web-search-mcp-rust --port 4126 &
