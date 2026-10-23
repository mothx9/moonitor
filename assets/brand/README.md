# Moonitor artwork

`moon-original.png` is the artwork supplied by the repository owner on 23 October
2026 for the app and repository identity. The original file is preserved unchanged,
including its alpha channel. This note does not assign an artist or infer a license.

The repository README references this original. The desktop embeds size-converted
128 px and 256 px PNG copies in `crates/moonitor-desktop/assets/`. They are decoded
once for the sidebar and window icon. Iced's `image-without-codecs` feature enables
raster rendering without enabling its default codec collection; the already-used
`image` crate supplies PNG decoding. Iced adds two small transitive dependencies,
`kamadak-exif` and `mutate_once`, recorded in Cargo.lock.

On macOS, `scripts/build-icons.sh` uses `sips` and `iconutil` to create the standard
16–1024 px icon representations in `.local/moonitor.icns`. Packaging installs it
as `Contents/Resources/moonitor.icns` and declares `CFBundleIconFile` before signing.
No background removal, recoloring or AI regeneration is applied.
