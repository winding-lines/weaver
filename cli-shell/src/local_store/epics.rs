//! Manages the epic information. Since this is a short lived client and the json_store is simple
//! we initiate the store on demand.
use super::json_store;
use weaver_error::*;


/// Save this epic name in the local storage,
/// does not change the current store.
pub fn save_epic(name: String) -> Result<()> {
    let mut store = json_store::JsonStore::init()?;
    store.content.active_epic = Some(name);
    store.save()
}

pub fn epic() -> Result<Option<String>> {
    let store = json_store::JsonStore::init()?;
    // let _ = self.json_store.fresh()?;
    Ok(store.content.active_epic.clone())
}

/// Return the active epic in a format that can be displayed, i.e. empty string for None.
pub fn epic_display() -> String {
    let store = match json_store::JsonStore::init() {
        Ok(s) => s,
        Err(_) => return String::new()
    };
    match store.content.active_epic {
        Some(ref s) => s.clone(),
        None => String::from(""),
    }
}
