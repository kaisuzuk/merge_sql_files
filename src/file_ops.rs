use chrono::{DateTime, Local};
use std::fs::File;
use std::io::Read;
use std::{error::Error, fs, path::PathBuf};

// 指定のディレクトリ直下のSQLファイルをマージする関数
pub fn merge_sql_files(
    dir: &str,
    get_time: impl Fn() -> DateTime<Local>,
) -> Result<String, Box<dyn Error>> {
    let mut merged = get_current_time(get_time);
    let files = get_files(dir)?;
    for path in files {
        if is_sql_file(&path) {
            if !is_utf8_file(&path) {
                return Err(From::from(format!(
                    "ファイルの文字コードがUTF-8ではありません: {}",
                    path.display()
                )));
            }
            merged.push('\n');
            let mut file = File::open(path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            merged.push_str(&contents);
        }
    }
    Ok(merged)
}

use alphanumeric_sort::compare_str;
use std::cmp::Ordering;

// ファイル名でソートする関数(自然順ソート)
fn sort_files(files: &mut Vec<PathBuf>) {
    files.sort_by(|a, b| {
        let a = a.file_name().unwrap().to_str().unwrap();
        let b = b.file_name().unwrap().to_str().unwrap();
        match compare_str(a, b) {
            std::cmp::Ordering::Less => Ordering::Less,
            std::cmp::Ordering::Equal => Ordering::Equal,
            std::cmp::Ordering::Greater => Ordering::Greater,
        }
    });
}

// 指定のディレクトリ直下のファイルをVectorに格納する関数
fn get_files(dir: &str) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_file() {
            files.push(path);
        }
    }
    sort_files(&mut files);
    Ok(files)
}

// 指定のファイルの拡張子が.sqlであるかを判定する関数('exec'で始まるファイル名は除外)
fn is_sql_file(path: &PathBuf) -> bool {
    let file_name = path.file_name().unwrap().to_str().unwrap();
    let ext = path.extension().unwrap().to_str().unwrap();

    if file_name.starts_with("exec") && ext == "sql" {
        return false;
    }
    ext == "sql"
}

// 指定のファイルの文字コードがUTF-8であるかを判定する関数
fn is_utf8_file(path: &PathBuf) -> bool {
    let mut file = File::open(path).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    match String::from_utf8(buf) {
        Ok(_) => true,
        Err(_) => false,
    }
}

// 現在時刻を取得する関数
fn get_current_time(get_time: impl Fn() -> DateTime<Local>) -> String {
    get_time().format("-- [%Y-%m-%d %H:%M:%S]\n").to_string()
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_merge_sql_files() {
        // テスト用のディレクトリとファイルを作成
        fs::create_dir_all("./test_sql").unwrap();
        let mut file2 = File::create("./test_sql/2.sql").unwrap();
        file2.write_all(b"SELECT * FROM table2;").unwrap();
        let mut file1 = File::create("./test_sql/1.sql").unwrap();
        file1.write_all(b"SELECT * FROM table1;").unwrap();

        let mock_time = || Local.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();

        // ファイルをマージ
        let merged = merge_sql_files("./test_sql", mock_time).unwrap();

        // マージされた内容を確認
        assert_eq!(
            merged,
            "-- [2020-01-01 00:00:00]\n\nSELECT * FROM table1;\nSELECT * FROM table2;"
        );

        // テスト用のディレクトリとファイルを削除
        fs::remove_dir_all("./test_sql").unwrap();
    }

    #[test]
    fn test_get_files() {
        // テスト用のディレクトリとファイルを作成
        fs::create_dir_all("./test_files").unwrap();
        let mut file1 = File::create("./test_files/file1.txt").unwrap();
        file1.write_all(b"Test file 1").unwrap();
        let mut file2 = File::create("./test_files/file2.txt").unwrap();
        file2.write_all(b"Test file 2").unwrap();

        // ファイルを取得
        let files = get_files("./test_files").unwrap();

        // ファイルの数とパスを確認
        assert_eq!(files.len(), 2);
        assert_eq!(files[0], PathBuf::from("./test_files/file1.txt"));
        assert_eq!(files[1], PathBuf::from("./test_files/file2.txt"));

        // テスト用のディレクトリとファイルを削除
        fs::remove_dir_all("./test_files").unwrap();
    }

    #[test]
    fn test_is_utf8_file_with_utf8_file() {
        // テスト用のディレクトリとファイルを作成
        fs::create_dir_all("./test_files").unwrap();
        // Create a test file with UTF-8 encoding
        let mut file = File::create("./test_files/utf8_file.sql").unwrap();
        file.write_all(b"SELECT * FROM table;").unwrap();

        // Check if the file is UTF-8 encoded
        let path = PathBuf::from("./test_files/utf8_file.sql");
        assert_eq!(is_utf8_file(&path), true);

        // Delete the test file
        fs::remove_file("./test_files/utf8_file.sql").unwrap();
    }

    #[test]
    fn test_is_utf8_file_with_non_utf8_file() {
        // テスト用のディレクトリとファイルを作成
        fs::create_dir_all("./test_files").unwrap();
        // Create a test file with non-UTF-8 encoding
        let mut file = File::create("./test_files/non_utf8_file.sql").unwrap();
        file.write_all(&[0xFF, 0xFE, 0x41, 0x00]).unwrap(); // UTF-16LE BOM and "A" character

        // Check if the file is UTF-8 encoded
        let path = PathBuf::from("./test_files/non_utf8_file.sql");
        assert_eq!(is_utf8_file(&path), false);

        // Delete the test file
        fs::remove_file("./test_files/non_utf8_file.sql").unwrap();
    }
}
