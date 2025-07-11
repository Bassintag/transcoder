use std::{
    fs::{self},
    io::{self},
    path::{Path, PathBuf},
};

pub mod discord;
pub mod ffmpeg;
pub mod ffprobe;
pub mod log;
pub mod utils;

const EXTENSIONS: &[&str] = &["mp4"];

pub fn list_movie_files(path: &Path, recursive: &bool) -> Result<Vec<PathBuf>, io::Error> {
    let mut movie_files = Vec::<PathBuf>::new();

    for result in fs::read_dir(path)? {
        let entry = result?;
        let file_type: fs::FileType = entry.file_type()?;
        let file_path = entry.path();

        if file_type.is_file() {
            if let Some(file_extension) = file_path.extension() {
                let file_extension = file_extension.to_str().unwrap();
                if EXTENSIONS
                    .iter()
                    .any(|ext| file_extension.eq_ignore_ascii_case(ext))
                {
                    movie_files.push(file_path);
                }
            }
        } else if file_type.is_dir() && *recursive {
            let children = list_movie_files(&file_path, recursive)?;
            movie_files.extend(children);
        }
    }

    Ok(movie_files)
}
