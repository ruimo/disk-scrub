use crate::control_file::ControlFile;

pub struct Report<'a> {
    pub added: Vec<&'a str>,
    pub removed: Vec<&'a str>,
    pub modified: Vec<&'a str>,
}

impl<'a> Report<'a> {
    pub fn new(from: &'a ControlFile, to: &'a ControlFile) -> Self {
        let mut added: Vec<&'a str> = vec![];
        let mut deleted: Vec<&'a str> = vec![];
        let mut modified: Vec<&'a str> = vec![];

        let mut from_idx = 0;
        let mut to_idx = 0;

        loop {
            if from.len() <= from_idx && to.len() <= to_idx { break; }
            else if from.len() <= from_idx && to_idx < to.len() {
                for i in to_idx..to.len() {
                    added.push(&to[i].file_path);
                }
                break;
            } else if from_idx < from.len() && to.len() <= to_idx {
                for i in from_idx..from.len() {
                    deleted.push(&from[i].file_path);
                }
                break;
            } else {
                let fc = &from[from_idx];
                let tc = &to[to_idx];
                if fc.file_path < tc.file_path {
                    deleted.push(&fc.file_path);
                    from_idx += 1;
                } else if tc.file_path < fc.file_path {
                    added.push(&tc.file_path);
                    to_idx += 1;
                } else {
                    if tc.sha256 != fc.sha256 {
                        modified.push(&tc.file_path);
                    }
                    from_idx += 1;
                    to_idx += 1;
                }
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