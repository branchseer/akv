#!/usr/bin/env zx

import fs from 'node:fs'

cd(path.resolve(__dirname, '..'))

const action = argv._[0]
if (action === 'style-check') {
    await $`cargo fmt --all -- --check`
    await $`cargo clippy --all-targets -- -D warnings`
} else if (action === 'test') {
    const cargo = ['cargo']
    if (process.platform === 'linux') {
        cargo.push('valgrind')
    }
    if (process.platform === 'win32') {
        process.env.RUSTFLAGS="-Ctarget-feature=+crt-static"
        process.env.OPENSSL_SRC_PERL = "C:/Strawberry/perl/bin/perl.exe"
        console.log(fs.statSync(process.env.OPENSSL_SRC_PERL))
    }
    await $`${cargo} test --release`
} else {
    console.error('Unrecognized action: ', action)
    process.exit(1)
}
