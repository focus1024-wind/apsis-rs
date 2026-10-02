# 内置的 MathJax

本目录保存一份 MathJax 副本，使生成的文档能够**离线**渲染数学公式，不向任何
内容分发网络发起请求。`docs/mathjax.html` 负责加载该文件，构建文档时
`crates/apsis-rs/build.rs` 会把它复制到 `target/doc/vendor/mathjax/`。

| 字段 | 值 |
| --- | --- |
| 包 | `mathjax` |
| 版本 | 3.2.2 |
| 文件 | `es5/tex-svg-full.js` |
| 来源 | `https://cdn.jsdelivr.net/npm/mathjax@3.2.2/es5/tex-svg-full.js` |
| 大小 | 2 275 113 字节 |
| SHA-256 | `a4354ff94fd868aea0cc6eaaa79a57fd`（前 32 位十六进制） |
| 许可 | Apache-2.0，见本目录的 `LICENSE` |

`tex-svg-full.js` 是合并组件，打包了 TeX 输入、SVG 输出与字体数据，故不需要
在线字体，也不需要第二次请求。这正是选用它而非 HTML 输出的原因——后者需要
内置一整个字体文件目录。

要更新该副本，请下载同一 URL，并更新上表中的大小、SHA-256 与版本。
