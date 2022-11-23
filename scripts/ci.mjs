#!/usr/bin/env zx

process.env.VCPKG_ROOT = process.env.VCPKG_INSTALLATION_ROOT
cd(path.resolve(__dirname, '..'))

const action = argv._[0]

if (action === 'style-check') {
    await $`vcpkg install libarchive`
    await $`cargo fmt --all -- --check`
    await $`cargo clippy --all-targets -- -D warnings`
} else if (action === 'test') {

} else {
    console.error('Unrecognized action: ', action)
    process.exit(1)
}
