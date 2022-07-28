use similar::{TextDiff, ChangeTag};

use crate::control_file::ControlFile;

pub struct Report<'a> {
    pub added: Vec<&'a str>,
    pub removed: Vec<&'a str>,
    pub modified: Vec<&'a str>,
}

impl<'a> Report<'a> {
    pub fn new(from: &'a ControlFile, to: &'a ControlFile) -> Self {
        let from_files = from.files();
        let to_files = to.files();
        let diff = TextDiff::from_slices(&from_files, &to_files);
        let mut added: Vec<&'a str> = vec![];
        let mut deleted: Vec<&'a str> = vec![];
        let mut modified: Vec<&'a str> = vec![];

        for d in diff.iter_all_changes() {
            let file_path = d.value();
            match d.tag() {
                ChangeTag::Equal =>
                  if from.get(file_path).unwrap().sha256 != to.get(file_path).unwrap().sha256 {
                    modified.push(file_path)
                  }
                ChangeTag::Delete => deleted.push(file_path),
                ChangeTag::Insert => added.push(file_path),
            }
        }

        Self { added, removed: deleted, modified }
    }
}

#[cfg(test)]
mod tests {
    use crate::control_file::{ControlFile, ControlFileEntry, str_hash};

    use super::Report;

    #[test]
    fn can_create_report() {
        let from = ControlFile {
            entries: vec![
                ControlFileEntry {
                    file_path: "ABC".to_owned(),
                    sha256: str_hash("ABC"),
                },
                ControlFileEntry {
                    file_path: "DEF".to_owned(),
                    sha256: str_hash("DEF"),
                },
                ControlFileEntry {
                    file_path: "EFG".to_owned(),
                    sha256: str_hash("EFG"),
                },
            ]
        };

        let to = ControlFile {
            entries: vec![
                ControlFileEntry {
                    file_path: "DEF".to_owned(),
                    sha256: str_hash("DEF0"),
                },
                ControlFileEntry {
                    file_path: "EFG".to_owned(),
                    sha256: str_hash("EFG"),
                },
                ControlFileEntry {
                    file_path: "XYZ".to_owned(),
                    sha256: str_hash("XYZ"),
                },
            ]
        };

        let report = Report::new(&from, &to);
        assert_eq!(report.added.len(), 1);
        assert_eq!(report.added[0], "XYZ");

        assert_eq!(report.removed.len(), 1);
        assert_eq!(report.removed[0], "ABC");

        assert_eq!(report.modified.len(), 1);
        assert_eq!(report.modified[0], "DEF");
    }
}