use token_sentinel::PhantomScanner;

#[test]
fn test_phantom_package_detection() {
    let scanner = PhantomScanner::with_known_dependencies(&["tokio", "serde", "pydantic"]);

    // Python code with 1 stdlib, 1 manifest, 1 hallucinated
    let py_code = "import os\nfrom pydantic import BaseModel\nimport hyper_fast_vector_db_pro\n";
    let warnings = scanner.scan_code(py_code);

    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].package_name, "hyper_fast_vector_db_pro");
    assert_eq!(warnings[0].language, "python");

    // Rust code with std, manifest (tokio), and 1 hallucinated crate
    let rs_code =
        "use std::sync::Arc;\nuse tokio::net::TcpStream;\nuse magic_crypto_cipher::Decryptor;\n";
    let rs_warnings = scanner.scan_code(rs_code);

    assert_eq!(rs_warnings.len(), 1);
    assert_eq!(rs_warnings[0].package_name, "magic_crypto_cipher");
    assert_eq!(rs_warnings[0].language, "rust");

    // Clean code -> 0 warnings
    let clean_code = "import json\nimport sys\n";
    assert!(scanner.scan_code(clean_code).is_empty());
}
