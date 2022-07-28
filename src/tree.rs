use std::{path::Path, io::{Error, ErrorKind}, fs};

pub fn list_recursive<P: AsRef<Path>>(dir: P) -> Result<Vec<String>, Error> {
    if ! dir.as_ref().exists() { return Err(ErrorKind::NotFound.into()); }
    if ! dir.as_ref().is_dir() { return Err(Error::new(ErrorKind::Other, "Not a directory.")); }
    let mut ret: Vec<String> = vec![];

    fn f<P0: AsRef<Path>, P1: AsRef<Path>>(root: P0, dir: P1, ret: &mut Vec<String>) -> Result<(), Error> {
        for e in fs::read_dir(dir)? {
            let path = e?.path();
            if path.is_dir() {
                f(root.as_ref(), path.as_path(), ret)?
            } else {
                ret.push(path.strip_prefix(root.as_ref()).unwrap().to_string_lossy().to_string());
            }
        }
    
        Ok(())
    }

    f(dir.as_ref(), dir.as_ref(), &mut ret).map(|_| ret)
}

#[cfg(test)]
mod tests {
    use std::fs::{File, self};
    use std::io::ErrorKind;
    use std::path::Path;
    use tempfile::tempdir;

    use super::list_recursive;

    #[test]
    fn can_read_single_dir() {
        let tmp_dir = tempdir().unwrap();
        File::create(tmp_dir.path().join("foo0.txt")).unwrap();
        File::create(tmp_dir.path().join("foo1.txt")).unwrap();

        let root = tmp_dir.into_path();
        let mut list = list_recursive(&root).unwrap();
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
        let mut list = list_recursive(&root).unwrap();
        list.sort();
        
        assert_eq!(list.len(), 2);
        assert_eq!(list[0], "foo/foo1.txt");
        assert_eq!(list[1], "foo0.txt");
    }

    #[test]
    fn can_treat_empty() {
        let tmp_dir = tempdir().unwrap();

        let root = tmp_dir.into_path();
        let list = list_recursive(&root).unwrap();
        
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn can_treat_non_exitent() {
        let root = Path::new("non_exitent");
        assert_eq!(list_recursive(&root).err().unwrap().kind(), ErrorKind::NotFound);
    }

    #[test]
    fn can_treat_file() {
        let tmp_dir = tempdir().unwrap();
        let file_path_buf = tmp_dir.path().join("foo0.txt");
        let file_path = file_path_buf.as_path();
        File::create(file_path).unwrap();

        assert_eq!(list_recursive(file_path).is_err(), true);
    }
}