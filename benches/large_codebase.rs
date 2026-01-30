//! Benchmark for analyzing large codebases.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use diedeadcode::analysis::Analyzer;
use diedeadcode::config::Config;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_test_files(dir: &std::path::Path, count: usize) {
    for i in 0..count {
        let content = format!(
            r#"
export function func{}() {{
    return {};
}}

export const value{} = func{}();

interface Type{} {{
    field: number;
}}
"#,
            i, i, i, i, i
        );
        let path = dir.join(format!("file{}.ts", i));
        std::fs::write(path, content).unwrap();
    }

    // Create an entry point
    let mut entry_content = String::from("// Entry point\n");
    for i in 0..count / 10 {
        entry_content.push_str(&format!("import {{ func{} }} from './file{}';\n", i, i));
        entry_content.push_str(&format!("func{}();\n", i));
    }
    std::fs::write(dir.join("index.ts"), entry_content).unwrap();

    // Create package.json
    std::fs::write(
        dir.join("package.json"),
        r#"{"name": "test", "main": "index.ts"}"#,
    )
    .unwrap();
}

fn benchmark_small(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    create_test_files(temp_dir.path(), 10);

    let config = Config::default();
    let path = temp_dir.path().to_path_buf();

    c.bench_function("analyze_10_files", |b| {
        b.iter(|| {
            let mut analyzer = Analyzer::new(config.clone(), path.clone()).unwrap();
            black_box(analyzer.analyze(None).unwrap())
        })
    });
}

fn benchmark_medium(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    create_test_files(temp_dir.path(), 100);

    let config = Config::default();
    let path = temp_dir.path().to_path_buf();

    c.bench_function("analyze_100_files", |b| {
        b.iter(|| {
            let mut analyzer = Analyzer::new(config.clone(), path.clone()).unwrap();
            black_box(analyzer.analyze(None).unwrap())
        })
    });
}

criterion_group!(benches, benchmark_small, benchmark_medium);
criterion_main!(benches);
