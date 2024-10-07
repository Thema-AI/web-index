# Thema web index

Thema's web index is a queryable store of the data fetched from the web, and
metadata about the fetching attempt.

The formal definition of the interface and storage backend can be found in the
[latest standard](standard/latest/standard.md).

**TODO update below when the library is finished: THIS IS STALE**

# Return types

The query interfaces return `Vec<Option<Page>>` in rust and `list[Page | None]`
in python.

# Examples

!!! TODO

    update when we have actual code

To fetch the latest page for any of a thousand urls, synchronously:

```python
from web_index.read_only import query
from web_index.models import SimplePageQuery

urls = ["http://example.com"] * 1_000 # replace with real data
queries = [SimplePageQuery(url=url) for url in urls]
results = query(queries)
print(results)
```

The same, asynchronously (note that the underlying rust code always runs in an
asynchronous loop: the api here is just to free up the python event loop to do
something else):
```python
from web_index.read_only import async_query
from web_index.models import SimplePageQuery

urls = ["http://example.com"] * 1_000 # replace with real data
queries = [SimplePageQuery(url=url) for url in urls]
results = await async_query(queries)
print(results)
```

## Development with nix

### Shell
To enter a devshell with everything available:

```shell
nix develop
```

Unless you have modified the template, this will use the declared nightly
toolchain. To use another toolchain, specify it explicitly:

```shell
nix develop .#msrv # uses msrv declared in Cargo.toml
nix develop .#stable
nix develop .#nightly # aliased to default
```

!!! Question "Isn't that commented out?"

    Most shells interpret # as a comment only when preceded by at least one
    whitespace character or when at the beginning of a line, so you can type the
    above without quotes.

### Direnv support

Run `direnv allow` to use enter default nix devshell whenever you cd to this repo.

### Normal workflow

Run `nix develop`, or enable direnv. Then use cargo as normal:

```shell
cargo add foo
cargo test
cargo run -- --help
cargo build --release
```

As a convenience, the release output dir is added to `$PATH`, so you can run the
built release artifact directly.

### Installing the nix package

A nix package is defined, so you can use this flake directly in your system
(providing you can fetch the flake in the first place: note that Thema's code is
*private* so you can't simply add the repo to your system's `flake.nix`).

### Updating rust/deps

Use `nix flake update`.

### Building cross-platform static binaries

TODO

### Development without nix

This isn't really supported at the moment as nobody uses it, but the following
should work:

- Install any required system deps (normally you can skip this, and rustc will
  tell you if you need to do anything)
- Install the right rust version with e.g. `rustup` (a recent stable or nightly
  should be fine)
- Use cargo as normal

If anyone wants to hack on any of this code and finds nix a problem, they should
[contact John](mailto:john@thema.ai).
