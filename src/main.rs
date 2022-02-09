mod data;

use std::{
    fs, panic,
    path::{Path, PathBuf},
};

use indicatif::{MultiProgress, ProgressBar, ProgressIterator, ProgressStyle};

use lazy_static::lazy_static;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Error {
    AssetsFolderNotFound(Vec<PathBuf>),
    IndexesFolderNotFound(Vec<PathBuf>),
    InvalidIndexFile(PathBuf),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::AssetsFolderNotFound(v) => {
                let mut ps = String::new();
                v.iter()
                    .for_each(|p| ps.push_str(&format!("{}, ", p.to_string_lossy())));
                f.write_fmt(format_args!(
                    "Could not find assets folder. Tried {}",
                    ps.trim_end_matches(", ")
                ))
            }
            Error::IndexesFolderNotFound(v) => {
                let mut ps = String::new();
                v.iter()
                    .for_each(|p| ps.push_str(&format!("{}, ", p.to_string_lossy())));
                f.write_fmt(format_args!(
                    "Could not find indexes folder. Tried {}",
                    ps.trim_end_matches(", ")
                ))
            }
            Error::InvalidIndexFile(p) => {
                f.write_fmt(format_args!("Invalid index file {}", p.display()))
            }
        }
    }
}

lazy_static! {
    static ref MULTI_PROGRESS_BAR: MultiProgress = MultiProgress::new();
}

fn main() -> Result<()> {
    let running_spinner = MULTI_PROGRESS_BAR.add(ProgressBar::new_spinner());
    let progress_thread_handle = std::thread::Builder::new()
        .name("Progress thread".into())
        .spawn(|| progress_thread_fn(&MULTI_PROGRESS_BAR))?;
    let assets_path = PathBuf::from("assets/");
    if !fs::metadata(&assets_path)?.is_dir() {
        return Err(Error::AssetsFolderNotFound(vec![assets_path]).into());
    }
    extract_assets(&assets_path)?;
    running_spinner.finish_and_clear();
    match progress_thread_handle.join() {
        Ok(v) => v,
        Err(e) => panic::resume_unwind(e),
    }?;

    Ok(())
}

fn progress_thread_fn(mp: &MultiProgress) -> Result<()> {
    mp.join_and_clear()?;
    Ok(())
}

fn extract_assets(path: impl AsRef<Path>) -> Result<()> {
    let indexes_path = path.as_ref().join("indexes");
    if !indexes_path.is_dir() {
        return Err(Error::IndexesFolderNotFound(vec![indexes_path]).into());
    }
    let iter = indexes_path.read_dir()?.flatten().collect::<Vec<_>>();
    let pb = MULTI_PROGRESS_BAR.add(ProgressBar::new(iter.len() as u64));
    for index_entry in iter.iter().progress_with(pb) {
        let index_path = index_entry.path();
        let output_dir = path.as_ref().join("files").join(
            index_path
                .file_stem()
                .ok_or_else(|| Error::InvalidIndexFile(index_path.clone()))?,
        );
        fs::create_dir_all(&output_dir)?;
        if let Ok(index_string) = fs::read_to_string(&index_path) {
            let index: data::Index = serde_json::from_str(&index_string)?;

            let pb_tmp = ProgressBar::new(index.objects().len() as u64)
                .with_style(ProgressStyle::default_bar().template(
                    "{prefix} [{elapsed_precise}/{duration_precise}] {wide_bar} {pos}/{len} {msg}",
                ))
                .with_prefix("Extracting assets");

            let pb_iter = index
                .objects()
                .iter()
                .progress_with(MULTI_PROGRESS_BAR.add(pb_tmp));
            for (object_path, object) in pb_iter {
                let output_path = output_dir.join(object_path);
                let input_path = path.as_ref().join(format!(
                    "objects/{}/{}",
                    object.hash().chars().take(2).collect::<String>(),
                    object.hash()
                ));
                fs::create_dir_all(output_path.parent().expect("WTF??? Shouldn't happen..."))?;
                fs::copy(input_path, output_path)?;
            }
        }
    }

    Ok(())
}
