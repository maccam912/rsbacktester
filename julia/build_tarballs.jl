# Note that this script can accept some limited command-line arguments, run
# `julia build_tarballs.jl --help` to see a usage message.
using BinaryBuilder, Pkg

name = "RSBacktester"
version = v"0.1.0"

# Collection of sources required to complete build
sources = [
    GitSource("https://github.com/maccam912/rsbacktester.git", "b98718c7929a99556edf5119df90e18526f56727")
]

# Bash recipe for building across all platforms
script = raw"""
cd $WORKSPACE/srcdir
cd rsbacktester/
PKG_CONFIG_ALLOW_CROSS=1 cargo build --release

RUST_LIB="$(pwd)/target/${rust_target}/release/librsbacktester.so"
if [[ ${target} == *mingw32* ]]; then
    mv "$(pwd)/target/${rust_target}/release/rsbacktester.lib" "${RUST_LIB}"
fi

cp "${RUST_LIB}" "${prefix}"
"""

# These are the platforms we will build for by default, unless further
# platforms are passed in on the command line
platforms = [
    #Windows(:x86_64),
    #Linux(:i686, libc=:glibc),
    Linux(:x86_64, libc=:glibc),
    #Linux(:aarch64, libc=:glibc),
    #Linux(:armv7l, libc=:glibc, call_abi=:eabihf),
    #Linux(:powerpc64le, libc=:glibc),
]


# The products that we will ensure are always built
products = [
    LibraryProduct("librsbacktester", :librsbacktester)
]

# Dependencies that must be installed before this package can be built
dependencies = Dependency[
]

# Build the tarballs, and possibly a `build.jl` as well.
build_tarballs(ARGS, name, version, sources, script, platforms, products, dependencies; compilers = [:rust, :c])
