#!/usr/bin/env zx

import fs from 'node:fs'

cd(path.resolve(__dirname, '..'))

const action = argv._[0]
if (action === 'vcpkg') {
    await $`vcpkg install libarchive`
    if (process.platform === 'linux') {
        await $`vcpkg install liblzma`
    }
    fs.writeFileSync(process.env.GITHUB_ENV, 'VCPKG_ROOT=' + process.env.VCPKG_INSTALLATION_ROOT)
} else if (action === 'style-check') {
    await $`cargo fmt --all -- --check`
    await $`cargo clippy --all-targets -- -D warnings`
} else if (action === 'test') {
    let cargoSubcommand = ''
    if (process.platform === 'linux') {
        cargoSubcommand = 'valgrind'
    }
    await $`cargo ${cargoSubcommand} test --release`
} else {
    console.error('Unrecognized action: ', action)
    process.exit(1)
}
