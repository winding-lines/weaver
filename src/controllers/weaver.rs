use ::entities::Weaver;
use ::errors::*;
use super::file_utils;

/// Initialize the weaver application/global configuration.
pub fn weaver_init() -> Result<Weaver> {
    file_utils::app_folder().and_then(|mut path| {
        path.push("weaver.json");
        if path.exists() {
            file_utils::read_content(&path)
                .and_then(|content| Weaver::load_from_string(&content))
                .chain_err(|| "loading weaver config")
        } else {
            Ok(Default::default())
        }
    })
}



/// Save the weaver application/global configuration.
pub fn weaver_save(weaver: Weaver) -> Result<()> {
    let mut path = file_utils::app_folder()?;
    path.push("weaver.json");
    let content = weaver.to_str()?;
    file_utils::write_content(&path, &content)
}

/// Activate the milestone with the given name.
pub fn milestone_activate(name: String) -> Result<()> {
    let mut weaver = weaver_init()?;
    weaver.active_milestone = Some(name);
    weaver_save(weaver)
}
