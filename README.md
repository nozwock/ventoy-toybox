# Ventoy Toybox

[![Latest Version](https://img.shields.io/github/v/tag/nozwock/ventoy-toybox.svg?label=Version&sort=semver)](https://github.com/nozwock/ventoy-toybox/releases)

A desktop application consisting of some helper utilities for [ventoy][ventoy]

> Download latest release [**`here!`**][release]

## Preview

<img src="https://user-images.githubusercontent.com/57829219/198823849-7da17229-2c6c-4d57-a745-0b6ec48db12a.png" alt="preview image" width="512">

> **NOTE:** Prebuilt binaries are available for `amd64` Linux and Bindows

## Features

- Easy to use
- Browse through latest linux distro releases
- Download linux distro images via torrents
- Filter releases by distro or torrent file name
- Fetch updates for [ventoy][ventoy]
- Blazingly Fast
- Messy code
- Even more messier code courtesy of `rustfmt` refusing to work

> Release feeds are handled by [nozwock/ventoy-toybox-feed](https://github.com/nozwock/ventoy-toybox-feed)

## Usage

> Starting `v0.4.0`, release feeds and ventoy update packages are cached on disk. <br/>

- To get latest release feeds you have to press the refresh button in the `Browse OS Releases` tab otherwise the on-disk cache will be used.

## Building

Clone the repository using git and change to the local repository directory:

```bash
git clone https://github.com/nozwock/ventoy-toybox.git
cd ventoy-toybox
```

`Stable Rust` is required to build this project. Install it by following [rustup.rs](https://rustup.rs) instructions.

```bash
cargo build --release
```

## Known Issues

- Scaling issues on x11 KDE Plasma; UI being too big

## FAQs
1. Wait a minute...couldn't you have just made a simple script to fetch updates? What's the purpose of all this?
    - Well...you wouldn't be enjoying this blazingly fast and feature-packed app then, would you? Haha..ha..hah...

## Licenese

[MIT](https://choosealicense.com/licenses/mit/)

[ventoy]: https://github.com/ventoy/Ventoy
[release]: https://github.com/nozwock/ventoy-toybox/releases/latest

<!-- this is an older image -->
<!-- <img src="https://user-images.githubusercontent.com/57829219/195810407-7c3474c4-56c8-42b7-b9d2-b6f06571a6c0.png" width="512"> -->

<!-- ![ferris_32](https://user-images.githubusercontent.com/57829219/199549292-1ea8c0f3-127a-48da-873b-80e80cd989b3.png) -->
<!-- ![ferris_worried_32](https://user-images.githubusercontent.com/57829219/199549897-4e4378f9-7664-4b10-955b-4fe1e0fdea2b.png) -->
<!-- ![ferris_smile_64](https://user-images.githubusercontent.com/57829219/199553233-d04815be-4192-4349-a610-86004e81e5da.png) -->
