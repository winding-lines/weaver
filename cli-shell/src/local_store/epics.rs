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


/// Return the active epic.
pub fn epic() -> Result<Option<String>> {
    let store = json_store::JsonStore::init()?;
    // let _ = self.json_store.fresh()?;
    Ok(store.content.active_epic.clone())
}

