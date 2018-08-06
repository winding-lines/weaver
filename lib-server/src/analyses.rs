use lib_error::*;
use lib_goo::config::file_utils;
use std::path::PathBuf;
use walkdir::WalkDir;
use inflections::Inflect;

#[derive(Serialize, Deserialize)]
pub struct Analysis {
    pub name: String,
    pub file: String,
    pub path: PathBuf,
}

/// Load the pre-computed analyses in the given folder.
/// Each html file is assumed to contain an analysis.
pub fn load_analyses() -> Result<Vec<Analysis>> {

    // The base folder for analyses
    let path = analyses_folder()?;

    // Read it
    let mut out = Vec::new();
    for entry in WalkDir::new(path) {
        let entry = entry.chain_err(|| "listing analyses")?;
        let path = entry.path();
        if let Some(os_name) = path.file_name() {
            if let Some(file) = os_name.to_str() {
                if entry.file_type().is_file() && file.ends_with(".html") {
                    let len = file.len();
                    let name = file[.. len-5].to_title_case();
                    out.push(Analysis {
                        name: name.into(),
                        file: file.into(),
                        path: path.into(),
                    });
                }
            }
        }
    }
    Ok(out)
}

pub fn get_analysis(file: &str) -> Result<String> {
    let mut path = analyses_folder()?;
    path.push(file);
    file_utils::read_content(&path)
}

fn analyses_folder() -> Result<PathBuf> {
    let mut path = file_utils::app_folder()?;
    path.push("analyses");
    path.push("html");
    Ok(path)
}
