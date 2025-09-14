// src/main.rs
mod io;
mod detector;
mod validator;

use std::env;
use std::process;
use std::path::Path;
use detector::{FileDetector, FileType};
use validator::FileValidator;

fn main() {
    // コマンドライン引数を取得
    let args: Vec<String> = env::args().collect();

    // 引数の数をチェック
    if args.len() < 2 {
        eprintln!("Usage: {} <path>", args[0]);
        process::exit(1);
    }

    let path = &args[1];

    // 基準パスを正規化（絶対パスに変換）
    let base_path = match std::fs::canonicalize(path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error resolving path '{}': {}", path, e);
            process::exit(1);
        }
    };

    // パスのタイプを確認
    match io::check_path_type(path) {
        Ok(io::PathType::Directory) => {
            process_directory(path, &base_path);
        }
        Ok(io::PathType::File) => {
            process_file(path, &base_path, true); // 最初のファイルなのでtrue
        }
        Ok(io::PathType::Other) => {
            eprintln!("Unsupported path type: {}", path);
            process::exit(1);
        }
        Err(e) => {
            eprintln!("Error accessing path '{}': {}", path, e);
            process::exit(1);
        }
    }
}

fn process_directory(path: &str, base_path: &Path) {
    // ディレクトリの場合、最初に===で表示
    println!("================================================================================");
    println!("{}/", base_path.display());
    println!("================================================================================");

    match io::walk_directory(path, Some(10)) {
        Ok(files) => {
            for file in files {
                process_file(&file, base_path, false); // 子ファイルなのでfalse
            }
        }
        Err(e) => {
            eprintln!("Error walking directory '{}': {}", path, e);
        }
    }
}

fn process_file(path: &str, base_path: &Path, is_root: bool) {
    // バリデーション
    if let Err(e) = FileValidator::quick_validate(path) {
        eprintln!("Skipping file '{}': {}", path, e);
        return;
    }
    let display_path = if is_root {
        format!("================================================================================\n{}\n================================================================================", path)
    } else {
        let file_path = match std::fs::canonicalize(path) {
            Ok(p) => p,
            Err(_) => {
                eprintln!("Error resolving path: {}", path);
                return;
            }
        };

        let relative_path = match file_path.strip_prefix(base_path) {
            Ok(rel) => format!("./{}", rel.display()),
            Err(_) => path.to_string(), // フォールバック
        };

        format!("--------------------------------------------------------------------------------\n{}\n--------------------------------------------------------------------------------", relative_path)
    };

    // ファイルタイプ判定
    match FileDetector::detect_file_type(path) {
        Ok(FileType::Binary) => {
            println!("{}", display_path);
            println!("This is a binary file\n");
        }
        Ok(FileType::Text) => {
            // テキストファイルの内容を表示
            match io::read_file_content(path) {
                Ok(contents) => {
                    println!("{}", display_path);
                    println!("{}\n", contents);
                }
                Err(e) => {
                    eprintln!("Error reading file '{}': {}", path, e);
                }
            }
        }
        Err(e) => {
            eprintln!("Error detecting file type '{}': {}", path, e);
        }
    }
}