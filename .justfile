target-linux := "x86_64-unknown-linux-gnu"
target-win := "x86_64-pc-windows-gnu"
bin_name := "ventoy-toybox"
# have to hardcode it for now
version := "0.2.2"
target_name := replace(bin_name, "-", "_") + "-" + version + "-amd64"
dist_dir := "./target"

# build for linux & win64 on a linux host
default: _pre-build build-linux build-win _post-build

_pre-build:
    mkdir -p {{dist_dir}}
    rm -f {{dist_dir}}/{{replace(bin_name, "-", "_")}}-*

build-host:
    cargo b --release

build-linux:
    rustup target add {{target-linux}}
    cross b --release --target {{target-linux}}
    cp -T ./target/{{target-linux}}/release/{{bin_name}} {{dist_dir}}/{{target_name}}-linux.bin
    upx --best --lzma ./target/{{target-linux}}/release/{{bin_name}}
    cp -T ./target/{{target-linux}}/release/{{bin_name}} {{dist_dir}}/{{target_name}}-linux-upxed.bin

build-win:
    rustup target add {{target-win}}
    cross b --release --target {{target-win}}
    cp -T ./target/{{target-win}}/release/{{bin_name}}.exe {{dist_dir}}/{{target_name}}-windows.exe
# upx --best --lzma ./target/{{target_win}}/release/{{bin_name}}.exe

_post-build:
    cd {{dist_dir}} && \
    sha256sum {{target_name}}* > checksums.sha256sum