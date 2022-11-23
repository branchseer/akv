#!/usr/bin/env zx

import fs from 'node:fs'

cd(path.resolve(__dirname, '..'))

const action = argv._[0]
if (action === 'vcpkg') {
    if (process.platform === 'win32') {
        await $`vcpkg install libarchive:x64-windows-static-md`
    } else {
        await $`vcpkg install libarchive`
    }
    fs.writeFileSync(process.env.GITHUB_ENV, 'VCPKG_ROOT=' + process.env.VCPKG_INSTALLATION_ROOT)
} else if (action === 'style-check') {
    await $`cargo fmt --all -- --check`
    await $`cargo clippy --all-targets -- -D warnings`
} else if (action === 'test') {
    let cargo = 'cargo'
    if (process.platform === 'linux') {
        cargo = 'cargo valgrind'
    }
    await $`${cargo} test --release`
} else {
    console.error('Unrecognized action: ', action)
    process.exit(1)
}
