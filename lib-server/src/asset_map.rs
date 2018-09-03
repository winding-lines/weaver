use lib_error::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::RwLock;

#[derive(Default)]
pub struct AssetMap(RwLock<HashMap<String, Vec<u8>>>);

const CSS: &[u8] = include_bytes!("../web/dist/css/weaver.css");

impl AssetMap {
    pub fn build() -> AssetMap {
        let mut hm = HashMap::new();
        hm.insert("weaver.css".into(), CSS.into());
        AssetMap(RwLock::new(hm))
    }

    pub fn reload(&self) -> Result<String> {
        let mut css = vec![];
        let path = Path::new("lib-server/web/dist/css/weaver.css");
        File::open(&path)?
            .read_to_end(&mut css)
            .map_err(|e| format!("cert open {:?}", e))?;
        let mut guard = self
            .0
            .write()
            .map_err(|e| format!("lock asset hash map {:?}", e))?;
        guard.insert("weaver.css".into(), css.into());
        Ok("weaver.css".into())
    }

    pub fn asset(&self, name: &str) -> Result<Vec<u8>> {
        let guard = self
            .0
            .read()
            .map_err(|e| format!("lock asset hash map {:?}", e))?;
        match guard.get(name) {
            Some(v) => Ok(v.clone()),
            None => Err("missing asset".into()),
        }
    }
}
