use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub fn read_trimmed_lines(path: &Path) -> std::io::Result<Vec<Vec<u8>>> {
    let mut reader = BufReader::new(File::open(path)?);
    let mut lines = Vec::new();
    let mut buf = Vec::new();

    loop {
        buf.clear();
        if reader.read_until(b'\n', &mut buf)? == 0 {
            break;
        }
        let trimmed = buf.trim_ascii();
        if !trimmed.is_empty() {
            lines.push(trimmed.to_vec());
        }
    }

    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn keeps_non_utf8_lines_and_skips_empty() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("wordlist.bin");
        let mut file = File::create(&path).unwrap();
        file.write_all(b"password\n\n  spaced  \r\n\xff\xfe\ntail")
            .unwrap();

        let lines = read_trimmed_lines(&path).unwrap();

        assert_eq!(
            lines,
            vec![
                b"password".to_vec(),
                b"spaced".to_vec(),
                vec![0xff, 0xfe],
                b"tail".to_vec(),
            ]
        );
    }
}
