target-linux := "x86_64-unknown-linux-gnu"
target-win := "x86_64-pc-windows-gnu"
name := "ventoy-toybox"

# build for linux & win64 on a linux host
default: build-linux build-win

build-host:
    cargo b --release

build-linux:
    rustup target add {{target-linux}}
    cross b --release --target {{target-linux}}
    upx --best --lzma ./target/{{target-linux}}/release/{{name}}

build-win:
    rustup target add {{target-win}}
    cross b --release --target {{target-win}}
# upx --best --lzma ./target/{{target-win}}/release/{{name}}.exe