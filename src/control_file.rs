use std::{io::{Error, self, BufRead, ErrorKind, BufWriter, Read}, path::Path, fs::File, fmt::{Display, self}};
use std::io::Write;

use sha2::{Sha256, Digest};

use crate::{tree, io_error::IoError};

pub struct ControlFile {
    pub entries: Vec<ControlFileEntry>,    
}

impl ControlFile {
    pub fn empty() -> Self {
        Self { entries: vec![] }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let mut recs: Vec<ControlFileEntry> = vec![];
        let lines = io::BufReader::new(File::open(&path)?).lines();
        let mut line_no: usize = 1;
        for l in lines {
            let l = l?;
            match ControlFileEntry::parse(&l) {
                Err(parse_error) =>
                    return Err(Error::new(ErrorKind::Other, format!("{:?}({}): {} '{}'.", path.as_ref().to_str(), line_no, parse_error, l))),
                Ok(e) => recs.push(e),
            }
            line_no += 1;
        }

        Ok(Self { entries: recs })
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let mut writer = BufWriter::new(File::create(&path)?);
        for e in self.entries.iter() {
            write!(writer, "{}\n", e)?;
        }
        Ok(())
    }

    pub fn load_from_dir<P: AsRef<Path>>(dir: P) -> Result<Self, IoError> {
        let mut list = tree::list_recursive(&dir).map_err(
            |err| IoError { cause: err, message: "Cannot access directory.".to_owned(), path: Some(dir.as_ref().to_owned())}
        )?;
        list.sort();
        let mut recs = Vec::with_capacity(list.len());
        for f in list.iter() {
            recs.push(
                ControlFileEntry::from_file(&dir, f.clone()).map_err(
                    |err| IoError { cause: err, message: "Cannot read file.".to_owned(), path: Some(dir.as_ref().join(f).to_owned()) }
                )?
            );
        }
        Ok(Self { entries: recs, })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn files(&self) -> Vec<&str> {
        let mut ret = Vec::with_capacity(self.entries.len());
        for e in self.entries.iter() {
            let s: &str = &e.file_path;
            ret.push(s);
        }
        ret
    }

    pub fn get(&self, file_path: &str) -> Option<&ControlFileEntry> {
        match self.entries.binary_search_by_key(&file_path, |e| { &e.file_path }) {
            Err(_) => None,
            Ok(idx) => Some(&self.entries[idx]),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    InvalidColumnCount(usize),
    InvalidHashFormat(String),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &ParseError::InvalidColumnCount(count) => write!(f, "Invalid column count(={}) expected 2.", count),
            ParseError::InvalidHashFormat(s) => write!(f, "Invalid hash format '{:?}'.", s),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct ControlFileEntry {
    pub file_path: String,
    pub sha256: Vec<u8>,
}

impl ControlFileEntry {
    pub fn from_file<P: AsRef<Path>>(root: P, file_path: String) -> Result<Self, Error> {
        let path = root.as_ref().join(&file_path);

        Ok(
            Self {
                file_path, sha256: file_hash(&path, None)?
            }
        )
    }

    pub fn parse(inp: &str) -> Result<Self, ParseError> {
        let cols: Vec<&str> = inp.split("\t").collect();
        if cols.len() != 2 {
            return Err(ParseError::InvalidColumnCount(cols.len()));
        }
        let file_path = cols[0].to_owned();
        let sha256 = match hex::decode(cols[1]) {
            Err(_) => { return Err(ParseError::InvalidHashFormat(cols[1].to_owned())); },
            Ok(hash) => hash,
        };
        if sha256.len() != 32 {
            return Err(ParseError::InvalidHashFormat(cols[1].to_owned()));
        }

        Ok(
            ControlFileEntry {
                file_path: file_path.to_owned(), sha256,
            }
        )
    }
}

impl fmt::Display for ControlFileEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\t{}", self.file_path, hex::encode(&self.sha256))
    }
}

const READ_BUF_SIZE: usize = 16 * 1024;

fn file_hash(path: &Path, buf_size: Option<usize>) -> Result<Vec<u8>, Error> {
    let buf_size = buf_size.unwrap_or(READ_BUF_SIZE);
    let mut file = File::open(path)?;
    let mut buf = vec![0u8; buf_size];
    let mut hasher = Sha256::new();

    loop {
        let read_size = file.read(&mut buf)?;
        if read_size == 0 { break; }
        hasher.update(&buf[0..read_size]);
    }

    Ok(hasher.finalize().to_vec())
}

pub fn str_hash(s: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(s);
    hasher.finalize().to_vec()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Write;
    use std::{fs::File, os::unix::prelude::FileExt};
    use tempfile::tempdir;
    use crate::control_file::{ParseError, file_hash};
    use super::{ControlFileEntry, ControlFile};
    use super::str_hash;

    #[test]
    fn str_hash_works() {
        // $ echo -n 012 | sha256sum
        // bf6aaaab7c143ca12ae448c69fb72bb4cf1b29154b9086a927a0a91ae334cdf7

        assert_eq!(
            str_hash("012"),
            vec![
                0xbfu8, 0x6au8, 0xaau8, 0xabu8, 0x7cu8, 0x14u8, 0x3cu8, 0xa1u8,
                0x2au8, 0xe4u8, 0x48u8, 0xc6u8, 0x9fu8, 0xb7u8, 0x2bu8, 0xb4u8,
                0xcfu8, 0x1bu8, 0x29u8, 0x15u8, 0x4bu8, 0x90u8, 0x86u8, 0xa9u8,
                0x27u8, 0xa0u8, 0xa9u8, 0x1au8, 0xe3u8, 0x34u8, 0xcdu8, 0xf7u8,
            ]
        );
    }

    #[test]
    fn can_parse_simple_case() {
        let inp = "ABC\t00112233445566778899aabbccddeeff0112233445566778899aabbccddeeff0";
        let e = ControlFileEntry::parse(inp).unwrap();
        assert_eq!(e.file_path, "ABC");
        assert_eq!(
            e.sha256,
            vec![
                0u8, 0x11u8, 0x22u8, 0x33u8, 0x44u8, 0x55u8, 0x66u8, 0x77u8, 0x88u8, 0x99u8, 0xaau8, 0xbbu8, 0xccu8, 0xddu8, 0xeeu8, 0xffu8,
                0x01u8, 0x12u8, 0x23u8, 0x34u8, 0x45u8, 0x56u8, 0x67u8, 0x78u8, 0x89u8, 0x9au8, 0xabu8, 0xbcu8, 0xcdu8, 0xdeu8, 0xefu8, 0xf0u8,
            ]
        );
        assert_eq!(e.to_string(), inp);
    }

    #[test]
    fn invalid_format() {
        assert_eq!(ControlFileEntry::parse("A").err().unwrap(), ParseError::InvalidColumnCount(1));
        assert_eq!(ControlFileEntry::parse("A\tB\tC").err().unwrap(), ParseError::InvalidColumnCount(3));
        assert_eq!(
            ControlFileEntry::parse("ABC\tZ0112233445566778899aabbccddeeff0112233445566778899aabbccddeeff0").err().unwrap(),
            ParseError::InvalidHashFormat("Z0112233445566778899aabbccddeeff0112233445566778899aabbccddeeff0".to_owned())
        );
        assert_eq!(ControlFileEntry::parse(
            "ABC\t112233445566778899aabbccddeeff0112233445566778899aabbccddeeff0").err().unwrap(),
            ParseError::InvalidHashFormat("112233445566778899aabbccddeeff0112233445566778899aabbccddeeff0".to_owned())
        );
    }

    #[test]
    fn can_save_load() {
        let entries = vec![
            ControlFileEntry {
                file_path: "ABC".to_owned(),
                sha256: str_hash("ABC"),
            },
            ControlFileEntry {
                file_path: "DEF".to_owned(),
                sha256: str_hash("DEF"),
            },
        ];

        let tmp_dir = tempdir().unwrap();
        let path = tmp_dir.path().join("foo0.ctrl");
        let cf = ControlFile { entries };
        cf.save_to_file(&path).unwrap();

        let loaded = ControlFile::load_from_file(&path).unwrap();
        assert_eq!(loaded.entries, cf.entries);
    }

    #[test]
    fn simple_hash_calc() {
        // $ echo -n 012 | sha256sum
        // bf6aaaab7c143ca12ae448c69fb72bb4cf1b29154b9086a927a0a91ae334cdf7
        let tmp_dir = tempdir().unwrap();
        let path = tmp_dir.path().join("foo.txt");
        File::create(&path).unwrap()
            .write_all_at(&[0x30u8, 0x31u8, 0x32u8], 0).unwrap();

        assert_eq!(file_hash(&path, Some(2)).unwrap(), str_hash("012"));
    }

    #[test]
    fn read_from_file() {
        // $ echo -n 012 | sha256sum
        // bf6aaaab7c143ca12ae448c69fb72bb4cf1b29154b9086a927a0a91ae334cdf7
        let tmp_dir = tempdir().unwrap();
        let path = tmp_dir.path().join("foo.txt");
        File::create(&path).unwrap()
            .write_all_at(&[0x30u8, 0x31u8, 0x32u8], 0).unwrap();

        let cfe = ControlFileEntry::from_file(&tmp_dir, "foo.txt".to_owned()).unwrap();
        assert_eq!(cfe.file_path, "foo.txt");
        assert_eq!(cfe.sha256, str_hash("012"))
    }

    #[test]
    fn read_from_dir() {
        let tmp_dir = tempdir().unwrap();
        {
            let mut foo0 = File::create(tmp_dir.path().join("foo0.txt")).unwrap();
            foo0.write_all(b"012").unwrap();
        }
        fs::create_dir(tmp_dir.path().join("foo")).unwrap();
        {
            let mut foo1 = File::create(tmp_dir.path().join("foo/foo1.txt")).unwrap();
            foo1.write_all(b"ABC").unwrap();
        }

        let list = ControlFile::load_from_dir(&tmp_dir).unwrap();
        assert_eq!(list.len(), 2);
        let e = &list.entries[0];
        assert_eq!(e.file_path, "foo/foo1.txt");
        assert_eq!(e.sha256, str_hash("ABC"));

        let e = &list.entries[1];
        assert_eq!(e.file_path, "foo0.txt");
        assert_eq!(e.sha256, str_hash("012"));
    }

    #[test]
    fn can_retrieve_file_list() {
        let cf = ControlFile {
            entries: vec![
                ControlFileEntry {
                    file_path: "ABC".to_owned(),
                    sha256: str_hash("ABC"),
                },
                ControlFileEntry {
                    file_path: "DEF".to_owned(),
                    sha256: str_hash("DEF"),
                },
            ]
        };
        assert_eq!(cf.files(), vec!["ABC", "DEF"]);
    }

    #[test]
    fn can_get_by_file_path() {
        let cf = ControlFile {
            entries: vec![
                ControlFileEntry {
                    file_path: "ABC".to_owned(),
                    sha256: str_hash("ABC"),
                },
                ControlFileEntry {
                    file_path: "foo/DEF".to_owned(),
                    sha256: str_hash("DEF"),
                },
            ]
        };
        assert_eq!(cf.get("ABC"), Some(&cf.entries[0]));
        assert_eq!(cf.get("foo/DEF"), Some(&cf.entries[1]));
        assert_eq!(cf.get("foo"), None);
        assert_eq!(cf.get("DEF"), None);
    }
}