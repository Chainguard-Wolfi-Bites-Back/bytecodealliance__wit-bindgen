//! Bindings generation support for Rust with the Component Model.
//!
//! This crate is a bindings generator for [WIT] and the [Component Model].
//! Users are likely interested in the [`generate!`] macro which actually
//! generates bindings. Otherwise this crate provides any runtime support
//! necessary for the macro-generated code.
//!
//! [WIT]: https://component-model.bytecodealliance.org/design/wit.html
//! [Component Model]: https://component-model.bytecodealliance.org/

#![no_std]

/// Generate bindings for an input WIT document.
///
/// This macro is the bread-and-butter of the `wit-bindgen` crate. The macro
/// here will parse [WIT] as input and generate Rust bindings to work with the
/// `world` that's specified in the [WIT]. For a primer on WIT see [this
/// documentation][WIT] and for a primer on worlds see [here][worlds].
///
/// [WIT]: https://component-model.bytecodealliance.org/design/wit.html
/// [worlds]: https://component-model.bytecodealliance.org/design/worlds.html
///
/// This macro takes as input a [WIT package] as well as a [`world`][worlds]
/// within that package. It will then generate a Rust function for all `import`s
/// into the world. If there are any `export`s then a Rust `trait` will be
/// generated for you to implement. The macro additionally takes a number of
/// configuration parameters documented below as well.
///
/// Basic invocation of the macro can look like:
///
/// ```
/// use wit_bindgen::generate;
/// # macro_rules! generate { ($($t:tt)*) => () }
///
/// generate!();
/// ```
///
/// This will parse a WIT package in the `wit` folder adjacent to your project's
/// `Cargo.toml` file. Within this WIT package there must be precisely one
/// `world` and that world will be the one that has bindings generated for it.
/// All other options remain at their default values (more on this below).
///
/// If your WIT package has more than one `world`, or if you want to select a
/// world from the dependencies, you can specify a world explicitly:
///
/// ```
/// use wit_bindgen::generate;
/// # macro_rules! generate { ($($t:tt)*) => () }
///
/// generate!("my-world");
/// generate!("wasi:cli/imports");
/// ```
///
/// This form of the macro takes a single string as an argument which is a
/// "world specifier" to select which world is being generated. As a single
/// string, such as `"my-world"`, this selects the world named `my-world` in the
/// package being parsed in the `wit` folder. The longer form specification
/// `"wasi:cli/imports"` indicates that the `wasi:cli` package, located in the
/// `wit/deps` folder, will have a world named `imports` and those bindings will
/// be generated.
///
/// If your WIT package is located in a different directory than one called
/// `wit` then it can be specified with the `in` keyword:
///
/// ```
/// use wit_bindgen::generate;
/// # macro_rules! generate { ($($t:tt)*) => () }
///
/// generate!(in "./my/other/path/to/wit");
/// generate!("a-world" in "../path/to/wit");
/// ```
///
/// The full-form of the macro, however, takes a braced structure which is a
/// "bag of options":
///
/// ```
/// use wit_bindgen::generate;
/// # macro_rules! generate { ($($t:tt)*) => () }
///
/// generate!({
///     world: "my-world",
///     path: "../path/to/wit",
///     // ...
/// });
/// ```
///
/// For documentation on each option, see below.
///
/// ## Debugging output to `generate!`
///
/// While `wit-bindgen` is tested to the best of our ability there are
/// inevitably bugs and issues that arise. These can range from bad error
/// messages to misconfigured invocations to bugs in the macro itself. To assist
/// with debugging these situations the macro recognizes an environment
/// variable:
///
/// ```shell
/// export WIT_BINDGEN_DEBUG=1
/// ```
///
/// When set the macro will emit the result of expansion to a file and then
/// `include!` that file. Any error messages generated by `rustc` should then
/// point to the generated file and allow you to open it up, read it, and
/// inspect it. This can often provide better context to the error than rustc
/// provides by default with macros.
///
/// It is not recommended to set this environment variable by default as it will
/// cause excessive rebuilds of Cargo projects. It's recommended to only use it
/// as necessary to debug issues.
///
/// ## Options to `generate!`
///
/// The full list of options that can be passed to the `generate!` macro are as
/// follows. Note that there are no required options, they all have default
/// values.
///
///
/// ```
/// use wit_bindgen::generate;
/// # macro_rules! generate { ($($t:tt)*) => () }
///
/// generate!({
///     // The name of the world that bindings are being generated for. If this
///     // is not specified then it's required that the package selected
///     // below has a single `world` in it.
///     world: "my-world",
///
///     // Path to parse WIT and its dependencies from. Defaults to the `wit`
///     // folder adjacent to your `Cargo.toml`.
///     path: "../path/to/wit",
///
///     // Enables passing "inline WIT". If specified this is the default
///     // package that a world is selected from. Any dependencies that this
///     // inline WIT refers to must be defined in the `path` option above.
///     //
///     // By default this is not specified.
///     inline: "
///         world my-world {
///             import wasi:cli/imports;
///
///             export my-run: func()
///         }
///     ",
///
///     // Additional traits to derive for all defined types. Note that not all
///     // types may be able to implement these traits, such as resources.
///     //
///     // By default this set is empty.
///     additional_derives: [PartialEq, Eq, Hash, Clone],
///
///     // If the `world` being generated has any exports, then this option is
///     // required. Each exported interface must have an entry here in addition
///     // to a `world` key if the world has any top-level exported functions.
///     //
///     // Each entry in this map points to a type in Rust. The specified type
///     // must implement the generated trait.
///     exports: {
///         // If the WIT world has top-level function exports, such as:
///         //
///         //      world my-world {
///         //          export foo: func();
///         //      }
///         //
///         // then this option specifies which type implements the world's
///         // exported functions.
///         world: MyWorld,
///
///         // For each exported interface from a world a key is additionally
///         // required. Each key must be a string of the form "a:b/c"
///         // specifying the "WIT path" to the interface. For example:
///         //
///         //      package my:package;
///         //
///         //      interface my-interface {
///         //          foo: func();
///         //      }
///         //
///         //      world my-world {
///         //          export my-interface;
///         //          export wasi:random/insecure-seed;
///         //      }
///         //
///         // this would require these fields to be specified:
///         "my:package/my-interface": MyInterface,
///         "wasi:random/insecure-seed": MyInsecureSeed,
///
///         // If an unnamed interface is used then the export's name is the key
///         // to use:
///         //
///         //      world my-world {
///         //          export foo: interface {
///         //              some-func: func();
///         //          }
///         //      }
///         //
///         // would require:
///         "foo": MyFoo,
///     },
///
///     // When generating bindings for imports it might be the case that
///     // bindings were already generated in a different crate. For example
///     // if your world refers to WASI types then the `wasi` crate already
///     // has generated bindings for all WASI types and structures. In this
///     // situation the key `with` here can be used to use those types
///     // elsewhere rather than regenerating types.
///     //
///     // The `with` key here only works for interfaces referred to by imported
///     // functions. Additionally it only supports replacing types at the
///     // interface level at this time.
///     //
///     // When an interface is specified here no bindings will be generated at
///     // all. It's assumed bindings are fully generated upstream. This is an
///     // indicator that any further references to types defined in these
///     // interfaces should use the upstream paths specified here instead.
///     //
///     // Any unused keys in this map are considered an error.
///     with: {
///         "wasi:io/poll": wasi::io::poll,
///     },
///
///     // An optional list of function names to skip generating bindings for.
///     // This is only applicable to imports and the name specified is the name
///     // of the function.
///     skip: ["foo", "bar", "baz"],
///
///     // Configuration of how Rust types are generated.
///     //
///     // This option will change how WIT types are mapped to Rust types. There
///     // are a number of ways this can be done depending on the context. For
///     // example a Rust `&str` is suitable to pass to an imported function but
///     // an exported function receives a `String`. These both represent the
///     // WIT type `string`, however.
///     //
///     // Type generation becomes extra-significant when aggregates come into
///     // play (such as a WIT `record` or `variant`), especially when the
///     // aggregate is used both in an imported function and exported one.
///     //
///     // There are three modes of ownership, documented here, but only one
///     // can be specified.
///     //
///     // The default mode is "Owning" meaning that all Rust types will by
///     // default contain their owned containers. For example a `record` with
///     // a `string` will map to a Rust `struct` containing a `String`. This
///     // maximizes the chance that types can be shared between imports and
///     // exports but can come at a cost where calling an import may require
///     // more allocations than necessary.
///     ownership: Owning,
///
///     // The second mode of ownership is "Borrowing". This mode then
///     // additionally has a boolean flag indicating whether duplicate types
///     // should be generated if necessary.
///     //
///     // This mode will prefer using borrowed values in Rust to represent WIT
///     // values where possible. For example if the argument to an imported
///     // function is a record-with-a-string then in Rust that will generate a
///     // `struct` with a lifetime parameter storing `&'a str`.
///     //
///     // The `duplicate_if_necessary` flag will cause duplicate types to be
///     // generated when a WIT type is used both in an import and export. In
///     // this situation one will be called `FooParam` and one will be called
///     // `FooResult` (where `foo` is the WIT name).
///     //
///     // It's generally recommended to not turn this on unless performance
///     // requires it. Even if so, please feel free to open an issue on the
///     // `wit-bindgen` repository to help improve the default "Owning" use
///     // case above if possible.
///     ownership: Borrowing { duplicate_if_necessary: false },
///
///     // This will suffix the custom section containing component type
///     // information with the specified string. This is not required by
///     // default but if the same world is generated in two different locations
///     // in the crate then one bindings generation location will need this
///     // suffix to avoid having the custom sections corrupt each other.
///     type_section_suffix: "suffix",
///
///     // Configures the path to the `wit-bindgen` crate itself. By default
///     // this is `wit_bindgen` assuming that your crate depends on the
///     // `wit-bindgen` crate itself.
///     runtime_path: "path::to::wit_bindgen",
///
///     // Configure where the `bitflags` crate is located. By default this
///     // is `wit_bindgen::bitflags` which already reexports `bitflags` for
///     // you.
///     bitflags_path: "path::to::bitflags",
///
///     // Indicates that instead of `&str` and `String` the `&[u8]` and
///     // `Vec<u8>` types should be used. Only intended for cases where
///     // compiled size is of the utmost concern as this can avoid pulling in
///     // UTF-8 validation.
///     raw_strings,
///
///     // Emits `#[cfg(feature = "std")]` around `impl Error for ... {}` blocks
///     // for generated types. This is a niche option that is only here to
///     // support the standard library itself depending on this crate one day.
///     std_feature,
///
///     // Force a workaround to be emitted for pre-Rust-1.69.0 modules to
///     // ensure that libc ctors run only once.
///     run_ctors_once_workaround: true,
/// });
/// ```
///
/// [WIT package]: https://component-model.bytecodealliance.org/design/packages.html
#[cfg(feature = "macros")]
pub use wit_bindgen_rust_macro::generate;

// Re-export `bitflags` so that we can reference it from macros.
#[doc(hidden)]
pub use bitflags;

/// For more information about this see `./ci/rebuild-libcabi-realloc.sh`.
#[cfg(feature = "realloc")]
mod cabi_realloc;

mod pre_wit_bindgen_0_20_0;

#[doc(hidden)]
pub mod rt {
    /// This function is called from generated bindings and will be deleted by
    /// the linker. The purpose of this function is to force a reference to the
    /// symbol `cabi_realloc` to make its way through to the final linker
    /// command line. That way `wasm-ld` will pick it up, see it needs to be
    /// exported, and then export it.
    ///
    /// For more information about this see `./ci/rebuild-libcabi-realloc.sh`.
    pub fn maybe_link_cabi_realloc() {
        #[cfg(target_family = "wasm")]
        {
            #[cfg(feature = "realloc")]
            extern "C" {
                fn cabi_realloc(
                    old_ptr: *mut u8,
                    old_len: usize,
                    align: usize,
                    new_len: usize,
                ) -> *mut u8;
            }
            // Force the `cabi_realloc` symbol to be referenced from here. This
            // is done with a `#[used]` Rust `static` to ensure that this
            // reference makes it all the way to the linker before it's
            // considered for garbage collection. When the linker sees it it'll
            // remove this `static` here (due to it not actually being needed)
            // but the linker will have at that point seen the `cabi_realloc`
            // symbol and it should get exported.
            #[cfg(feature = "realloc")]
            #[used]
            static _NAME_DOES_NOT_MATTER: unsafe extern "C" fn(
                *mut u8,
                usize,
                usize,
                usize,
            ) -> *mut u8 = cabi_realloc;
        }
    }

    /// NB: this function is called by a generated function in the
    /// `cabi_realloc` module above. It's otherwise never explicitly called.
    ///
    /// For more information about this see `./ci/rebuild-libcabi-realloc.sh`.
    #[cfg(feature = "realloc")]
    pub unsafe fn cabi_realloc(
        old_ptr: *mut u8,
        old_len: usize,
        align: usize,
        new_len: usize,
    ) -> *mut u8 {
        use alloc::Layout;

        let layout;
        let ptr = if old_len == 0 {
            if new_len == 0 {
                return align as *mut u8;
            }
            layout = Layout::from_size_align_unchecked(new_len, align);
            alloc::alloc(layout)
        } else {
            debug_assert_ne!(new_len, 0, "non-zero old_len requires non-zero new_len!");
            layout = Layout::from_size_align_unchecked(old_len, align);
            alloc::realloc(old_ptr, layout, new_len)
        };
        if ptr.is_null() {
            // Print a nice message in debug mode, but in release mode don't
            // pull in so many dependencies related to printing so just emit an
            // `unreachable` instruction.
            if cfg!(debug_assertions) {
                alloc::handle_alloc_error(layout);
            } else {
                #[cfg(target_arch = "wasm32")]
                core::arch::wasm32::unreachable();
                #[cfg(not(target_arch = "wasm32"))]
                unreachable!();
            }
        }
        return ptr;
    }

    pub use crate::pre_wit_bindgen_0_20_0::*;
}
