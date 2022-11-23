#!/usr/bin/env zx

process.env.VCPKG_ROOT = process.env.VCPKG_INSTALLATION_ROOT
cd(path.resolve(__dirname, '..'))

await $`vcpkg install libarchive`

const action = argv._[0]
if (action === 'style-check') {
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
