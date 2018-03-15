use ::display::NameWithId;
use ::errors::*;
use ::store::Store;
use diesel::prelude::*;
use super::backends::models::Action;

pub fn history<T: AsRef<str>>(store: &mut Store, _epic: &Option<T>) -> Result<Vec<NameWithId>> {
    use super::backends::schema::actions::dsl::*;

    let entries = actions.load::<Action>(store.sqlite_connection())
        .chain_err(|| "fetching actions")?;
    let mut out = Vec::new();

    for entry in entries {
        let ref _comment = entry.annotation.as_ref().map_or("", String::as_str);
        let ref _tags_ = entry.tags.as_ref().map_or("", String::as_str);
        out.push(NameWithId {
            name: entry.command,
            kind: entry.kind,
            id: entry.id.unwrap_or(0) as usize,
        });
    }

    Ok(out)
}