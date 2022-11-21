#!/usr/bin/env zx

cd(path.resolve(__dirname, '..'))

await $`cargo test`
await $`cargo clippy -- -D warnings`