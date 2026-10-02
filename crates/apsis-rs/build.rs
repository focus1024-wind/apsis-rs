//! Copies the vendored `MathJax` into the documentation root.
//!
//! The generated HTML loads `MathJax` from `vendor/mathjax` under the
//! documentation root so that a formula renders without network access. The
//! header at `docs/mathjax.html` computes the path to that root at run time,
//! but the file itself has to be present, and rustdoc has no option to copy an
//! extra static file. This build script therefore places it there.
//!
//! The copy is a few hundred kilobytes, runs once per build, and does nothing
//! when the vendored directory is absent, so it is harmless for builds that do
//! not touch the documentation.
//!
//! 把内置的 `MathJax` 复制到文档根。
//!
//! 生成的 HTML 从文档根下的 `vendor/mathjax` 加载 MathJax，使公式无需网络访问
//! 即可渲染。`docs/mathjax.html` 处的头部在运行时计算到该根的路径，但文件
//! 本身必须存在，而 rustdoc 没有复制额外静态文件的选项。故本构建脚本将其
//! 放置到那里。
//!
//! 该复制只有几百千字节，每次构建运行一次，且在内置目录不存在时什么都不做，
//! 故对不涉及文档的构建无害。

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    // The facade crate is at `<workspace>/crates/apsis-rs`, so the workspace
    // root is two levels up.
    // 门面 crate 位于 `<workspace>/crates/apsis-rs`，故工作区根在其上两级。
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("cargo sets this"));
    let Some(workspace) = manifest.parent().and_then(Path::parent) else {
        return;
    };

    let source = workspace.join("docs").join("vendor").join("mathjax");
    if !source.is_dir() {
        return;
    }

    // The target directory is the default one unless the caller moved it.
    // 除非调用方移动了目标目录，否则它就是默认位置。
    let target = match env::var("CARGO_TARGET_DIR") {
        Ok(dir) => {
            let dir = PathBuf::from(dir);
            if dir.is_absolute() {
                dir
            } else {
                workspace.join(dir)
            }
        }
        Err(_) => workspace.join("target"),
    };
    let destination = target.join("doc").join("vendor").join("mathjax");

    if fs::create_dir_all(&destination).is_err() {
        return;
    }
    let Ok(entries) = fs::read_dir(&source) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let _ = fs::copy(&path, destination.join(entry.file_name()));
        }
    }

    println!("cargo:rerun-if-changed=../../docs/vendor/mathjax");
}
