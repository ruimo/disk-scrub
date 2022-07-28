use std::path::Path;

use clap::Parser;
use control_file::ControlFile;
use report::Report;

mod tree;
mod control_file;
mod report;

#[derive(Parser)]
#[clap(author, version, about, long_about = Some("Checks file integrity."))]
struct Cli {
    /// Target directory to preform integrity check. Will be checked recursively.
    #[clap(value_parser)]
    target_dir: String,

    /// Control file location. Will be updated accoring to the contents under the target directory.
    /// If the tool aborts, this file will not be changed.
    #[clap(short = 'f', long, value_parser, default_value_t = String::from("Controlfile"))]
    control_file: String,    
}

fn main() {
    let cli = Cli::parse();
    let control_file = Path::new(&cli.control_file);
    let target_dir = Path::new(&cli.target_dir);

    let to = perform(&control_file, &target_dir, |report| {
        println!("Summary:");
        println!("  Added files: {}", report.added.len());
        println!("  Removed files: {}", report.removed.len());
        println!("  Modified files: {}", report.modified.len());

        println!("");
        println!("Details:");
        println!("[Added files]");
        for f in report.added.iter() {
            println!("  {:?}", f);
        }

        println!("[Removed files]");
        for f in report.removed.iter() {
            println!("  {:?}", f);
        }

        println!("[Modified files]");
        for f in report.modified.iter() {
            println!("  {:?}", f);
        }
    });

    to.save_to_file(&control_file).unwrap();
}

fn perform<F, P, O>(control_file: F, target_dir: P, out: O) -> ControlFile
    where F: AsRef<Path>, P: AsRef<Path>, O: FnOnce(&Report)
{
    let from = 
        if ! control_file.as_ref().exists() {
            ControlFile::empty()
        } else {
            ControlFile::load_from_file(&control_file).unwrap()
        };

    let to = ControlFile::load_from_dir(&target_dir).unwrap();
    
    out(&Report::new(&from, &to));

    to
}

#[cfg(test)]
mod tests {
    use std::{fs::{File, self}, io::Write};
    use tempfile::tempdir;
    use crate::{perform};

    #[test]
    fn tiny_case() {
        let tmp_dir = tempdir().unwrap();
        let ctrl_dir = tempdir().unwrap();
        {
            let mut foo0 = File::create(tmp_dir.path().join("foo0.txt")).unwrap();
            foo0.write_all(b"012").unwrap();
        }
        fs::create_dir(tmp_dir.path().join("foo")).unwrap();
        {
            let mut foo1 = File::create(tmp_dir.path().join("foo/foo1.txt")).unwrap();
            foo1.write_all(b"ABC").unwrap();
        }
        {
            let mut foo2 = File::create(tmp_dir.path().join("foo/foo2.txt")).unwrap();
            foo2.write_all(b"DEF").unwrap();
        }

        let from = ctrl_dir.path().join("Controlfile");
        let mut report_called = false;

        let to = perform(&from, &tmp_dir, |report| {
            report_called = true;
            assert_eq!(report.added.len(), 3);
            assert_eq!(report.removed.len(), 0);
            assert_eq!(report.modified.len(), 0);
        });

        assert_eq!(report_called, true);
        to.save_to_file(&from).unwrap();

        {
            let mut foo1 = File::create(tmp_dir.path().join("foo/foo1.txt")).unwrap();
            foo1.write_all(b"ZZZ").unwrap();
        }
        {
            let mut foo2 = File::create(tmp_dir.path().join("foo/foo3.txt")).unwrap();
            foo2.write_all(b"YYY").unwrap();
        }
        fs::remove_file(tmp_dir.path().join("foo/foo2.txt")).unwrap();

        report_called = false;
        perform(&from, &tmp_dir, |report| {
            report_called = true;
            assert_eq!(report.added.len(), 1);
            assert_eq!(report.added[0], "foo/foo3.txt");

            assert_eq!(report.removed.len(), 1);
            assert_eq!(report.removed[0], "foo/foo2.txt");

            assert_eq!(report.modified.len(), 1);
            assert_eq!(report.modified[0], "foo/foo1.txt");
        });

    }

}