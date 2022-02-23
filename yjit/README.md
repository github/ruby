# Rust YJIT

⚠️ *Warning:* this project is a work-in-progress prototype. You may find crashes,
subtle bugs, lack of documentation, questionable designs, broken builds,
nondescript commit messages, etc.

## Getting Rust tools

I used the [recommended installation method][rust-install] on an Intel-based
MacBook and it went smoothly. Rust provides first class [support][editor-tools]
for many editors which might interest you.

## Useful commands

Cargo is your friend. Make sure you are in this folder (`cd yjit` from
repository root).

```sh
cargo build                       # build the static library
cargo test                        # run tests
cargo test --features disassembly # run additional tests that use the optional libcapstone for verification
cargo doc --document-private-items --open # build documentation site and open it in your browser
cargo fmt                         # reformat the source code (idempotent)
```

## Generating C bindings with bindgen

To generate C bindings for the YJIT Rust code, run:

```sh
CC=clang ./configure --enable-yjit=dev
make -j yjit-bindgen
```

This will generate/update `yjit/src/cruby_bindings.inc.rs`. Avoid manually editing this file
as it could be automatically regenerated at a later time.
If you need to manually add C bindings, add them to `yjit/cruby.rs` instead.

## Are you going to use Rust in other parts of CRuby?

No.

[rust-install]: https://www.rust-lang.org/tools/install
[editor-tools]: https://www.rust-lang.org/tools

## Running YJIT on M1

It is possible to run YJIT on an M1 via Rosetta.  You can find the most basic
instructions below, but there are a few caveats listed further down.

First, install Rosetta:

```
$ softwareupdate --install-rosetta
```

Now any command can be run with Rosetta via the `arch` command line tool.

Then you can start your shell in an x86 environment:

```
$ arch -x86_64 zsh
```

You can double check your current architecture via the `arch` command:

```
$ arch -x86_64 zsh
aaron@tc-lan-adapter ~ % arch
i386
aaron@tc-lan-adapter ~ % 
```

While in your i386 shell, install Cargo and Homebrew, then hack away!

## M1 Caveats

1. You must install a version of Homebrew for each architecture
2. Cargo will install in $HOME/.cargo by default, and I don't know a good way to change architectures after install
3. `dev` won't work if you have i386 Homebrew installed on an M1

If you use Fish shell you can [read this link](https://tenderlovemaking.com/2022/01/07/homebrew-rosetta-and-ruby.html) for information on making the dev environment easier.
