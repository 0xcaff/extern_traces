//! Utilities for snapshot testing. Snapshot tests compares the current output of a function
//! against a snapshot of the expected output, which is saved to disk. This is useful for
//! finding unintended changes.

use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;
use std::{fs, io};

/// Generates snapshots for files specified by `glob_pattern`, storing snapshots in `snapshots_base`
/// with paths relative to `input_base`.
///
/// ## Example
/// ```
/// use snapshot_test_utils::generate_snapshots;
///
/// # #[allow(unused_variables)]
/// fn debug(input: &[u8]) -> String {
///     // function under test
///     # "".to_string()
/// }
///
/// # #[allow(unused_variables)]
/// fn display(input: &[u8]) -> String {
///     // function under test
///     # "".to_string()
/// }
///
/// fn main() -> Result<(), anyhow::Error> {
///     generate_snapshots(
///         "test_data/captured/**/*.sb",
///         "test_data/captured",
///         "test_data/snapshots",
///         |input, collector| {
///             collector.result(
///                 "debug",
///                 &debug(&input),
///             )?;
///
///             // Supports multiple result kinds
///             collector.result(
///                 "display",
///                 &display(&input),
///             )?;
///
///             Ok(())
///         }
///     )?;
///
///     Ok(())
/// }
/// ```
///
/// Creates the following file hierarchy:
///
/// ```text
/// └── test_data
///     ├── captured                        // input files
///     │   ├── FGNM00001
///     │   │   ├── clear.frag.sb
///     │   │   ├── cube.frag.sb
///     │   │   └── cube.vert.sb
///     │   └── embedded
///     │       └── full_screen.vert.sb
///     └── snapshots                       // generated files
///         ├── FGNM00001
///         │   ├── clear.frag.debug.snap
///         │   ├── clear.frag.display.snap
///         │   ├── cube.frag.debug.snap
///         │   ├── cube.frag.display.snap
///         │   ├── cube.vert.debug.snap
///         │   └── cube.vert.display.snap
///         └── embedded
///             ├── full_screen.vert.debug.snap
///             └── full_screen.vert.display.snap
/// ```
pub fn generate_snapshots<
    F: Fn(Vec<u8>, &mut dyn TestResultCollector) -> Result<(), anyhow::Error>,
>(
    glob_pattern: &str,
    input_base: &str,
    snapshots_base: &str,
    run_test: F,
) -> Result<(), anyhow::Error> {
    for input_path in glob::glob(glob_pattern)? {
        let input_path = input_path?;

        let relative_path = input_path.strip_prefix(input_base)?;

        let snapshot_folder = Path::new(snapshots_base).join(relative_path);
        fs::create_dir_all(snapshot_folder.parent().unwrap())?;

        let mut collector = TestResultCollectorFn {
            collector_fn: |key, value| {
                let mut path = snapshot_folder.clone();
                path.set_extension(&format!("{}.snap", key));

                update_snapshot(&path, value)?;

                Ok(())
            },
        };

        let input = fs::read(input_path)?;

        run_test(input, &mut collector)?;
    }

    Ok(())
}

fn update_snapshot(snapshot_path: impl AsRef<Path>, expected: &str) -> Result<bool, io::Error> {
    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(snapshot_path)?;

    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;

    let bytes = expected.as_bytes();
    file.write_all(bytes)?;
    file.set_len(bytes.len() as _)?;

    Ok(expected == file_contents)
}

pub trait TestResultCollector {
    fn result(&mut self, key: &str, value: &str) -> Result<(), anyhow::Error>;
}

struct TestResultCollectorFn<F: FnMut(&str, &str) -> Result<(), anyhow::Error>> {
    collector_fn: F,
}

impl<F: FnMut(&str, &str) -> Result<(), anyhow::Error>> TestResultCollector
    for TestResultCollectorFn<F>
{
    fn result(&mut self, key: &str, value: &str) -> Result<(), anyhow::Error> {
        (self.collector_fn)(key, value)
    }
}
