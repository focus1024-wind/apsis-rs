# Vendored MathJax

This directory holds a copy of MathJax so that the generated documentation
can render mathematical formulas **offline**, without fetching anything from
a content delivery network. `docs/mathjax.html` loads the file, and
`crates/apsis-rs/build.rs` copies it into `target/doc/vendor/mathjax/` when
the documentation is built.

| Field | Value |
| --- | --- |
| Package | `mathjax` |
| Version | 3.2.2 |
| File | `es5/tex-svg-full.js` |
| Source | `https://cdn.jsdelivr.net/npm/mathjax@3.2.2/es5/tex-svg-full.js` |
| Size | 2 275 113 bytes |
| SHA-256 | `a4354ff94fd868aea0cc6eaaa79a57fd` (first 32 hex digits) |
| Licence | Apache-2.0, see `LICENSE` in this directory |

`tex-svg-full.js` is the combined component that bundles the TeX input and
the SVG output with the font data, so it needs no web font and no second
request. That is why this build was chosen over the HTML output, which would
require vendoring a directory of font files.

To refresh the copy, download the same URL and update the size, the SHA-256,
and the version above.
