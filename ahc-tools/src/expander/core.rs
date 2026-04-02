use anyhow::Context;
use std::{
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::Command,
};

const ROOT_MACROS: &[&str] = &["params_impl", "dump", "dumpln", "perf"];

#[derive(Debug, Clone)]
pub struct BundleRequest {
    pub solution_path: PathBuf,
    pub ahc_library_path: Option<PathBuf>,
    pub rustfmt: bool,
}

impl Default for BundleRequest {
    fn default() -> Self {
        Self {
            solution_path: PathBuf::from("."),
            ahc_library_path: None,
            rustfmt: true,
        }
    }
}

pub fn bundle_solution(request: &BundleRequest) -> anyhow::Result<String> {
    let mut solution = expand_solution(&request.solution_path)?;
    solution = solution.replace("use ahc_library::", "use crate::ahc_library::");

    if let Some(ahc_library_path) = &request.ahc_library_path {
        let library = expand_ahc_library(ahc_library_path)?;
        solution.push_str(&library);
    }

    solution = replace_macros(solution);

    if request.rustfmt {
        Ok(apply_rustfmt(solution))
    } else {
        Ok(solution)
    }
}

fn extract_modules(lib_file: &Path) -> anyhow::Result<Vec<String>> {
    let file = fs::File::open(lib_file)
        .with_context(|| format!("failed to open module list file: {}", lib_file.display()))?;
    let reader = BufReader::new(file);
    let mut modules = Vec::new();

    for line in reader.lines() {
        let line = line.with_context(|| format!("failed to read line: {}", lib_file.display()))?;
        let Some(mod_name) = extract_module_name(&line, "pub mod ") else {
            continue;
        };

        if mod_name == "test" {
            continue;
        }

        modules.push(mod_name);
    }

    Ok(modules)
}

fn read_modules_without_test(mod_name: &str, file_path: &Path) -> anyhow::Result<String> {
    let file = fs::File::open(file_path)
        .with_context(|| format!("failed to open module file: {}", file_path.display()))?;
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    content.push_str(&format!("pub mod {} {{\n", mod_name));

    let mut line = String::new();
    let mut is_test = false;
    loop {
        line.clear();
        let bytes = reader
            .read_line(&mut line)
            .with_context(|| format!("failed to read line: {}", file_path.display()))?;
        if bytes == 0 {
            break;
        }

        if line.starts_with("#[test]") || line.starts_with("#[cfg(test)]") {
            is_test = true;
        } else if is_test {
            if line.starts_with('}') {
                is_test = false;
            }
        } else {
            let Some(line) = rewrite_root_macro_use(&line, false) else {
                continue;
            };
            content.push_str("\t\t\t");
            content.push_str(&line);
        }
    }

    content.push_str("}\n\n");
    Ok(content)
}

fn replace_macros(mut content: String) -> String {
    for macro_name in ROOT_MACROS {
        content = content.replace(
            &format!("use crate::ahc_library::{{{macro_name}}};"),
            &format!("use crate::{macro_name};"),
        );
        content = content.replace(
            &format!("use crate::ahc_library::{macro_name};"),
            &format!("use crate::{macro_name};"),
        );
    }

    content
}

fn expand_ahc_library(ahc_library_path: &Path) -> anyhow::Result<String> {
    let src_dir = ahc_library_path.join("src");
    let lib_file = src_dir.join("lib.rs");
    let modules = extract_modules(&lib_file)?;

    let mut content = String::from("\npub mod ahc_library {");
    for module in modules {
        let sub_dir = src_dir.join(&module);
        let mod_file = sub_dir.join("mod.rs");
        let mod_content = fs::read_to_string(&mod_file)
            .with_context(|| format!("failed to read module file: {}", mod_file.display()))?;

        content.push_str(&format!("\npub mod {} {{", module));
        for line in mod_content.lines() {
            let Some(mod_name) = extract_module_name(line, "pub mod ") else {
                content.push_str(line);
                content.push('\n');
                continue;
            };

            if mod_name == "test" {
                continue;
            }

            content.push_str(&read_modules_without_test(
                &mod_name,
                &sub_dir.join(format!("{mod_name}.rs")),
            )?);
        }

        content.push_str("}\n");
    }
    content.push_str("}\n");

    Ok(content.replace("crate::", "crate::ahc_library::"))
}

fn expand_solution(solution_dir: &Path) -> anyhow::Result<String> {
    let src_dir = solution_dir.join("src");
    let main_file = src_dir.join("main.rs");
    let main_content = fs::read_to_string(&main_file).with_context(|| {
        format!(
            "failed to read solution entry file: {}",
            main_file.display()
        )
    })?;
    let mut content = String::new();

    for line in main_content.lines() {
        let Some(line) = rewrite_root_macro_use(line, true) else {
            continue;
        };

        let Some(mod_name) = extract_module_name(&line, "mod ") else {
            content.push_str(&line);
            content.push('\n');
            continue;
        };

        content.push_str(&read_modules_without_test(
            &mod_name,
            &src_dir.join(format!("{mod_name}.rs")),
        )?);
    }

    Ok(content)
}

fn rewrite_root_macro_use(line: &str, is_main_file: bool) -> Option<String> {
    let trimmed = line.trim();
    let leading_whitespace = &line[..line.len() - line.trim_start().len()];

    for macro_name in ROOT_MACROS {
        if trimmed == format!("use ahc_library::{macro_name};") {
            if is_main_file {
                return None;
            }

            return Some(format!("{leading_whitespace}use crate::{macro_name};\n"));
        }
    }

    Some(line.to_string())
}

fn extract_module_name(line: &str, prefix: &str) -> Option<String> {
    let start = line.find(prefix)? + prefix.len();
    let rest = &line[start..];
    let end = rest.find(';')?;
    let mod_name = rest[..end].trim();

    if mod_name.is_empty()
        || !mod_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return None;
    }

    Some(mod_name.to_string())
}

fn apply_rustfmt(content: String) -> String {
    let Ok(temp_file) = tempfile::Builder::new().suffix(".rs").tempfile() else {
        return content;
    };

    if fs::write(temp_file.path(), &content).is_err() {
        return content;
    }

    let Ok(status) = Command::new("rustfmt").arg(temp_file.path()).status() else {
        return content;
    };

    if !status.success() {
        return content;
    }

    fs::read_to_string(temp_file.path()).unwrap_or(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::Path};

    fn write(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("failed to create parent directories");
        }
        fs::write(path, content).expect("failed to write test file");
    }

    #[test]
    fn test_bundles_solution_with_library() {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let solution_dir = temp_dir.path().join("solution");
        let library_dir = temp_dir.path().join("ahc_library");

        write(
            &solution_dir.join("src/main.rs"),
            r#"mod helper;

use ahc_library::algo::score;

fn main() {
    helper::run();
    let _ = score();
}
"#,
        );
        write(
            &solution_dir.join("src/helper.rs"),
            r#"pub fn run() {
    println!("run");
}

#[cfg(test)]
mod tests {
    #[test]
    fn ignored() {}
}
"#,
        );

        write(
            &library_dir.join("src/lib.rs"),
            "pub mod algo;\npub mod test;\n",
        );
        write(
            &library_dir.join("src/algo/mod.rs"),
            "use crate::{params_impl};\npub mod score;\n",
        );
        write(
            &library_dir.join("src/algo/score.rs"),
            r#"pub fn score() -> i32 {
    42
}

#[test]
fn ignored() {}
"#,
        );

        let bundled = bundle_solution(&BundleRequest {
            solution_path: solution_dir,
            ahc_library_path: Some(library_dir),
            rustfmt: false,
        })
        .expect("bundle should succeed");

        assert!(bundled.contains("pub mod helper {"));
        assert!(bundled.contains("use crate::ahc_library::algo::score;"));
        assert!(bundled.contains("pub mod ahc_library {"));
        assert!(bundled.contains("pub mod algo {"));
        assert!(bundled.contains("use crate::params_impl;"));
        assert!(bundled.contains("pub mod score {"));
        assert!(!bundled.contains("fn ignored()"));
        assert!(!bundled.contains("pub mod test"));
    }

    #[test]
    fn test_keeps_nested_solution_modules_unexpanded() {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let solution_dir = temp_dir.path().join("solution");

        write(
            &solution_dir.join("src/main.rs"),
            r#"mod outer;

fn main() {
    outer::run();
}
"#,
        );
        write(
            &solution_dir.join("src/outer.rs"),
            r#"mod inner;

pub fn run() {
    inner::run();
}
"#,
        );
        write(&solution_dir.join("src/inner.rs"), "pub fn run() {}\n");

        let bundled = bundle_solution(&BundleRequest {
            solution_path: solution_dir,
            ahc_library_path: None,
            rustfmt: false,
        })
        .expect("bundle should succeed");

        assert!(bundled.contains("pub mod outer {"));
        assert!(bundled.contains("mod inner;"));
        assert!(!bundled.contains("pub mod inner {"));
    }

    #[test]
    fn test_root_macros_removed_from_main() {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let solution_dir = temp_dir.path().join("solution");

        write(
            &solution_dir.join("src/main.rs"),
            r#"use ahc_library::params_impl;

"#,
        );

        let bundled = bundle_solution(&BundleRequest {
            solution_path: solution_dir,
            ahc_library_path: None,
            rustfmt: false,
        })
        .expect("bundle should succeed");

        assert!(
            !bundled.contains("use ahc_library::params_impl;"),
            "bundle: {}",
            bundled
        );
        assert!(
            !bundled.contains("use crate::params_impl;"),
            "bundle: {}",
            bundled
        );
    }

    #[test]
    fn test_root_macros_removed_from_main_and_rewritten_in_modules() {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let solution_dir = temp_dir.path().join("solution");

        write(
            &solution_dir.join("src/main.rs"),
            r#"mod helper;

use ahc_library::params_impl;
use ahc_library::dump;

fn main() {
    helper::run();
}
"#,
        );
        write(
            &solution_dir.join("src/helper.rs"),
            r#"use ahc_library::params_impl;
use ahc_library::dump;

pub fn run() {}
"#,
        );

        let bundled = bundle_solution(&BundleRequest {
            solution_path: solution_dir,
            ahc_library_path: None,
            rustfmt: false,
        })
        .expect("bundle should succeed");

        assert!(!bundled.contains("use crate::ahc_library::{params_impl};"));
        assert!(!bundled.contains("use crate::ahc_library::dump;"));
        assert!(!bundled.contains("use ahc_library::{params_impl};"));
        assert!(!bundled.contains("use ahc_library::dump;"));
        assert_eq!(bundled.matches("use crate::params_impl;").count(), 1);
        assert_eq!(bundled.matches("use crate::dump;").count(), 1);
    }
}
