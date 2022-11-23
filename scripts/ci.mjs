#!/usr/bin/env zx

cd(path.resolve(__dirname, '..'))

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
