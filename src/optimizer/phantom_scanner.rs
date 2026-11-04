use std::collections::HashSet;

#[derive(Debug, Clone, serde::Serialize)]
pub struct PhantomWarning {
    pub language: String,
    pub package_name: String,
    pub statement: String,
    pub explanation: String,
}

pub struct PhantomScanner {
    py_stdlib: HashSet<&'static str>,
    rs_stdlib: HashSet<&'static str>,
    js_stdlib: HashSet<&'static str>,
    known_dependencies: HashSet<String>,
}

impl PhantomScanner {
    pub fn new() -> Self {
        let py_stdlib = [
            "os", "sys", "re", "math", "time", "datetime", "json", "typing", "collections",
            "itertools", "functools", "pathlib", "subprocess", "hashlib", "random", "logging",
            "urllib", "http", "socket", "asyncio", "threading", "multiprocessing", "abc",
            "contextlib", "dataclasses", "enum", "io", "copy", "struct", "shutil", "tempfile",
            "unittest", "argparse", "traceback", "gc", "warnings", "inspect", "uuid", "base64",
        ]
        .iter()
        .copied()
        .collect();

        let rs_stdlib = [
            "std", "core", "alloc", "crate", "super", "self",
        ]
        .iter()
        .copied()
        .collect();

        let js_stdlib = [
            "fs", "path", "http", "https", "url", "crypto", "os", "util", "events", "stream",
            "buffer", "child_process", "net", "tls", "dns", "assert", "zlib", "perf_hooks",
        ]
        .iter()
        .copied()
        .collect();

        Self {
            py_stdlib,
            rs_stdlib,
            js_stdlib,
            known_dependencies: HashSet::new(),
        }
    }

    pub fn with_known_dependencies(deps: &[&str]) -> Self {
        let mut scanner = Self::new();
        for d in deps {
            scanner.known_dependencies.insert(d.to_lowercase());
        }
        scanner
    }

    pub fn add_dependency(&mut self, dep: &str) {
        self.known_dependencies.insert(dep.to_lowercase());
    }

    /// Scan text (or code block) for potential hallucinated packages
    pub fn scan_code(&self, code: &str) -> Vec<PhantomWarning> {
        let mut warnings = Vec::new();

        for line in code.lines() {
            let trimmed = line.trim();

            // 1. Python imports
            if trimmed.starts_with("import ") {
                let pkg_part = trimmed.trim_start_matches("import ").trim();
                let pkg = pkg_part.split(',').next().unwrap_or(pkg_part).split('.').next().unwrap_or(pkg_part).split_whitespace().next().unwrap_or("");
                self.check_package("python", pkg, trimmed, &self.py_stdlib, &mut warnings);
            } else if trimmed.starts_with("from ") && trimmed.contains(" import ") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    let full_pkg = parts[1];
                    let root_pkg = full_pkg.split('.').next().unwrap_or(full_pkg);
                    if !root_pkg.starts_with('.') {
                        self.check_package("python", root_pkg, trimmed, &self.py_stdlib, &mut warnings);
                    }
                }
            }

            // 2. JavaScript / TypeScript imports
            if (trimmed.starts_with("import ") || trimmed.starts_with("import{") || trimmed.starts_with("import {")) && trimmed.contains("from ") {
                if let Some(quote_start) = trimmed.find("from ") {
                    let from_part = &trimmed[quote_start + 5..].trim();
                    let clean_from = from_part.trim_matches(|c| c == '\'' || c == '"' || c == ';' || c == ' ');
                    if !clean_from.starts_with('.') && !clean_from.starts_with('/') && !clean_from.starts_with('@') {
                        let root_pkg = clean_from.split('/').next().unwrap_or(clean_from);
                        self.check_package("javascript/typescript", root_pkg, trimmed, &self.js_stdlib, &mut warnings);
                    }
                }
            } else if trimmed.contains("require(") {
                if let Some(start) = trimmed.find("require(") {
                    let sub = &trimmed[start + 8..];
                    let clean = sub.split(')').next().unwrap_or("").trim_matches(|c| c == '\'' || c == '"' || c == ' ');
                    if !clean.starts_with('.') && !clean.starts_with('/') {
                        let root_pkg = clean.split('/').next().unwrap_or(clean);
                        self.check_package("javascript/typescript", root_pkg, trimmed, &self.js_stdlib, &mut warnings);
                    }
                }
            }

            // 3. Rust crate imports
            if trimmed.starts_with("use ") && !trimmed.starts_with("use std") && !trimmed.starts_with("use crate") && !trimmed.starts_with("use super") && !trimmed.starts_with("use self") {
                let sub = trimmed.trim_start_matches("use ").trim();
                let root_crate = sub.split("::").next().unwrap_or("").trim_matches(';');
                if !root_crate.is_empty() && !self.rs_stdlib.contains(root_crate) {
                    self.check_package("rust", root_crate, trimmed, &self.rs_stdlib, &mut warnings);
                }
            }
        }

        warnings
    }

    fn check_package(
        &self,
        lang: &str,
        pkg: &str,
        line: &str,
        stdlib: &HashSet<&'static str>,
        warnings: &mut Vec<PhantomWarning>,
    ) {
        let pkg_clean = pkg.trim();
        if pkg_clean.is_empty() {
            return;
        }

        let pkg_lower = pkg_clean.to_lowercase();

        if stdlib.contains(pkg_clean) || stdlib.contains(pkg_lower.as_str()) {
            return; // Valid stdlib module
        }

        if self.known_dependencies.contains(&pkg_lower) {
            return; // In project manifest
        }

        // Unknown package -> Potential Hallucination / Phantom Dependency!
        warnings.push(PhantomWarning {
            language: lang.to_string(),
            package_name: pkg_clean.to_string(),
            statement: line.to_string(),
            explanation: format!(
                "Package '{}' is NOT in project manifests or standard library. Verify it exists to prevent dependency confusion attack or broken build.",
                pkg_clean
            ),
        });
    }
}
