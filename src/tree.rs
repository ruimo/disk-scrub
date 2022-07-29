use std::{path::Path, io::{Error, ErrorKind}, fs};

use crate::{io_error::IoError, Cli, exclude::Exclude};

pub fn list_recursive<P: AsRef<Path>>(dir: P, cli: &Cli) -> Result<Vec<String>, IoError> {
    if ! dir.as_ref().exists() {
        return Err(
            IoError {
                cause: ErrorKind::NotFound.into(), message: "Not found.".to_owned(), path: Some(dir.as_ref().to_owned())
            }
        );
     }
    if ! dir.as_ref().is_dir() {
        return Err(
            IoError {
                cause: Error::new(ErrorKind::Other, "Not a directory."),
                message: "Not a directory".to_owned(),
                path: Some(dir.as_ref().to_owned()) }
        );
    }
    let mut ret: Vec<String> = vec![];
    let exclude = Exclude::new(cli.exclude.clone());

    fn f<P0: AsRef<Path>, P1: AsRef<Path>>(
        root: P0, dir: P1, ret: &mut Vec<String>, exclude: &Exclude
    ) -> Result<(), IoError> {
        if exclude.matches(&dir.as_ref().file_name().unwrap().to_string_lossy()) {
            return Ok(())
        }
        let read_dir = fs::read_dir(dir.as_ref()).map_err(|err|
            IoError {
                cause: err, message: "Cannot read directory.".to_owned(), path: Some(dir.as_ref().to_owned())
            }
        )?;
        for e in read_dir {
            let path = e.map_err(|err|
                IoError {
                    cause: err, message: "Cannot list entries in this directory.".to_owned(), path: Some(dir.as_ref().to_owned())
                }
            )?.path();
            if path.is_dir() {
                f(root.as_ref(), path.as_path(), ret, exclude)?
            } else {
                let name = path.file_name().unwrap().to_string_lossy();
                if ! exclude.matches(&name) {
                    ret.push(path.strip_prefix(root.as_ref()).unwrap().to_string_lossy().to_string());
                }
            }
        }
    
        Ok(())
    }

    f(dir.as_ref(), dir.as_ref(), &mut ret, &exclude).map(|_| ret)
}

#[cfg(test)]
mod tests {
    use std::fs::{File, self};
    use std::io::ErrorKind;
    use std::path::Path;
    use tempfile::tempdir;

    use crate::Cli;

    use super::list_recursive;

    #[test]
    fn can_read_single_dir() {
        let tmp_dir = tempdir().unwrap();
        File::create(tmp_dir.path().join("foo0.txt")).unwrap();
        File::create(tmp_dir.path().join("foo1.txt")).unwrap();

        let root = tmp_dir.into_path();
        let cli = Cli {
            control_file: "".to_owned(),
            exclude: vec![],
            target_dir: "".to_owned(),
        };
        let mut list = list_recursive(&root, &cli).unwrap();
        list.sort();

        assert_eq!(list.len(), 2);
        assert_eq!(list[0], "foo0.txt");
        assert_eq!(list[1], "foo1.txt");
    }

    #[test]
    fn can_read_recurse() {
        let tmp_dir = tempdir().unwrap();
        File::create(tmp_dir.path().join("foo0.txt")).unwrap();
        fs::create_dir(tmp_dir.path().join("foo")).unwrap();
        File::create(tmp_dir.path().join("foo/foo1.txt")).unwrap();

        let root = tmp_dir.into_path();
        let cli = Cli {
            control_file: "".to_owned(),
            exclude: vec![],
            target_dir: "".to_owned(),
        };
        let mut list = list_recursive(&root, &cli).unwrap();
        list.sort();
        
        assert_eq!(list.len(), 2);
        assert_eq!(list[0], "foo/foo1.txt");
        assert_eq!(list[1], "foo0.txt");
    }

    #[test]
    fn can_treat_empty() {
        let tmp_dir = tempdir().unwrap();
        let cli = Cli {
            control_file: "".to_owned(),
            exclude: vec![],
            target_dir: "".to_owned(),
        };

        let root = tmp_dir.into_path();
        let list = list_recursive(&root, &cli).unwrap();
        
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn can_treat_non_exitent() {
        let root = Path::new("non_exitent");
        let cli = Cli {
            control_file: "".to_owned(),
            exclude: vec![],
            target_dir: "".to_owned(),
        };
        assert_eq!(list_recursive(&root, &cli).err().unwrap().cause.kind(), ErrorKind::NotFound);
    }

    #[test]
    fn can_treat_file() {
        let tmp_dir = tempdir().unwrap();
        let file_path_buf = tmp_dir.path().join("foo0.txt");
        let file_path = file_path_buf.as_path();
        File::create(file_path).unwrap();
        let cli = Cli {
            control_file: "".to_owned(),
            exclude: vec![],
            target_dir: "".to_owned(),
        };

        assert_eq!(list_recursive(file_path, &cli).is_err(), true);
    }

    #[test]
    fn can_treat_exclude() {
        let tmp_dir = tempdir().unwrap();
        File::create(tmp_dir.path().join("foo0.txt")).unwrap();
        File::create(tmp_dir.path().join("0bar0.txt")).unwrap();
        fs::create_dir(tmp_dir.path().join("0foo")).unwrap();
        File::create(tmp_dir.path().join("0foo/foo1.txt")).unwrap();
        fs::create_dir(tmp_dir.path().join("bar")).unwrap();
        File::create(tmp_dir.path().join("bar/0foo0.txt")).unwrap();
        File::create(tmp_dir.path().join("bar/foo1.txt")).unwrap();

        let root = tmp_dir.into_path();
        let cli = Cli {
            control_file: "".to_owned(),
            exclude: vec!["0*".to_owned()],
            target_dir: "".to_owned(),
        };

        let mut list = list_recursive(&root, &cli).unwrap();
        list.sort();
        
        assert_eq!(list.len(), 2);
        assert_eq!(list[0], "bar/foo1.txt");
        assert_eq!(list[1], "foo0.txt");
   }
}