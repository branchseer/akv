#!/usr/bin/env zx

process.env.VCPKG_ROOT = process.env.VCPKG_INSTALLATION_ROOT
await $`vcpkg install libarchive`
if (process.platform === 'linux') {
    await $`vcpkg install liblmza`
}
