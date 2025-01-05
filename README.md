# Miscellaneous Rust libs

A collection of Rust crates, from simple utilities and extension traits to domain specific libraries, developed
mostly as prototypes for personal use.

They're on GitHub mainly for convenience, as Cargo supports git repositories as library paths.

If you need to use any of these, you can add the specific crate in the Cargo.toml dependency list, e.g.

```ini
# TODO: put the current commit ID in place of the "..." so that you can control when/if to update the dependency.
mutex_ext = { git = "https://github.com/cdellacqua/miscellaneous_libs.rs.git", rev = "..." }
```

If you'd like some of these libraries to become fully-fledged projects published on crates.io, feel free to open
an issue.
