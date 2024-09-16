
## Rust Optimal Transport
- Rust optimal transport depends on the `openblas-src` crate which (on Windows) requires an installation of `OpenBlas` using `vcpkg`.
- This installation is a relatively convoluted process, so here it is:
    - Clone the [vcpkg](https://github.com/Microsoft/vcpkg) repository
    - Execute `bootstrap-vcpkg.bat`
    - Add the root of the vcpkg repository to your PATH
    - Execute `vcpkg integrate install`
    - Install OpenBlas: `vcpkg install openblas --triplet x64-windows`  (Following the instructions on the [openblas-src](https://github.com/blas-lapack-rs/openblas-src) repo)
    - This (for me) gave a linking error. Download the release from the OpenBlas repository, and replace the files that vcpkg installed with these. Be sure to rename all `libopenblas.<...>` files to `openblas.<...>`.
        -  For more information, see [this issue](https://github.com/rust-ndarray/ndarray-linalg/issues/369)

