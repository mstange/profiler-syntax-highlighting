use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use profiler_syntax_highlighting_lib::SyntaxParsedFile;

fn main() {
    let path: PathBuf = std::env::args_os()
        .nth(1)
        .expect("supply a single path as the program argument").into();

    let mut f = File::open(&path).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();

    let extension = path.extension().and_then(OsStr::to_str).unwrap_or("");

    let line_count = s.lines().count();
    let mut file = SyntaxParsedFile::new(extension, s, Default::default());

    let range = if line_count <= 10 {
        println!("Printing all lines of {}:", path.to_string_lossy());
        0..line_count
    } else {
        let start = (line_count / 2) - 5;
        let end = start + 10;
        println!("Printing 10 lines ({} - {}) in the middle of {}:", start + 1, end, path.to_string_lossy());
        start..end
    };

    for i in range {
        println!("{:5}: {}", i + 1, file.html_for_line(i).as_deref().unwrap_or(""));
    }
}
