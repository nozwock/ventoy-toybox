target-linux := "x86_64-unknown-linux-gnu"
target-win := "x86_64-pc-windows-gnu"
bin_name := "ventoy-toybox"
# have to hardcode it for now
version := "0.2.1"
target_name := replace(bin_name, "-", "_") + "-" + version + "-amd64"
dist_dir := "./target"

# build for linux & win64 on a linux host
default: _pre-build build-linux build-win _post-build

@_CopyFileAs fpath tpath:
    cp {{fpath}} {{parent_directory(tpath)}}
    mv {{parent_directory(tpath)}}/{{file_name(fpath)}} {{parent_directory(tpath)}}/{{file_name(tpath)}}

_pre-build:
    rm -f {{dist_dir}}/{{target_name}}*
    mkdir -p {{dist_dir}}

build-host:
    cargo b --release

build-linux:
    rustup target add {{target-linux}}
    cross b --release --target {{target-linux}}
    just _CopyFileAs ./target/{{target-linux}}/release/{{bin_name}} {{dist_dir}}/{{target_name}}-linux.bin
    upx --best --lzma ./target/{{target-linux}}/release/{{bin_name}}
    just _CopyFileAs ./target/{{target-linux}}/release/{{bin_name}} {{dist_dir}}/{{target_name}}-linux-upxed.bin

build-win:
    rustup target add {{target-win}}
    cross b --release --target {{target-win}}
    just _CopyFileAs ./target/{{target-win}}/release/{{bin_name}}.exe {{dist_dir}}/{{target_name}}-windows.exe
# upx --best --lzma ./target/{{target_win}}/release/{{bin_name}}.exe

_post-build:
    cd {{dist_dir}} && \
    sha256sum {{target_name}}* > checksums.sha256sum