# Build

Kuutamod comes as a single binary and a optional cli `kuutamoctl` to inspect
kuutamod's state at runtime.

## Build with nix

We primarly develop kuutamod with nix and our development environment is based on it:

1. Install [nix](https://nix.dev/tutorials/install-nix)
2. Enable [flake support](https://xeiaso.net/blog/nix-flakes-1-2022-02-21) in nix:

```console
mkdir -p ~/.config/nix
echo 'experimental-features = nix-command flakes' >> ~/.config/nix/nix.conf
```

3. Build and run kuutamod:

```
nix run github:kuutamoaps/kuutamod -- --version
kuutamod 0.1.0
```

It's also possible to open a shell with development dependencies like this:

```
git clone https://github.com/kuutamoaps/kuutamod/
cd kuutamod
nix develop .#
```

The resulting shell allows to build kuutamod from nix like this:

```
nix-shell> cargo build
./target/debug/kuutamod --version
kuutamod 0.1.0
```

## Build without nix

Currently we are only testing `kuutamod` on Linux.

1. Download kuutamod i.e. with [git](https://git-scm.com/downloads)

```
git clone https://github.com/kuutamoaps/kuutamod/
```

2. Install will need `rustc` and `cargo` based on your [Linux distributions](https://www.rust-lang.org/learn/get-started).
   Due to dependencies on neard libraries, you will need a relativly new version
   of rust. In doubt, install `rustup` to get the latest rust version: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

3. Build kuutamod with `cargo`:

```
cd kuutamod
cargo build --release
./target/release/kuutamod --version
kuutamod 0.1.0
```
