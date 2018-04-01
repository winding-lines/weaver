use ::display::FormattedAction;
use ::errors::*;
use ::store::RealStore;
use diesel::prelude::*;
use super::backends::models::Action;
use super::backends::schema::actions::dsl::*;

pub fn history<T: AsRef<str>>(store: &mut RealStore, _epic: &Option<T>) -> Result<Vec<FormattedAction>> {
    let entries = actions.load::<Action>(store.sqlite_connection())
        .chain_err(|| "fetching actions")?;
    let mut out = Vec::new();

    for entry in entries {
        let ref _tags_ = entry.tags.as_ref().map_or("", String::as_str);
        out.push(FormattedAction {
            annotation: entry.annotation,
            id: entry.id.unwrap_or(0) as usize,
            epic: entry.epic,
            kind: entry.kind,
            name: entry.command,
            location: entry.location,
        });
    }

    Ok(out)
}

pub fn last_url(store: &mut RealStore) -> Result<Option<(String, String)>> {
    let entries = actions.limit(1)
        .load::<Action>(store.sqlite_connection())
        .chain_err(|| "loading last url")?;
    let first = entries.into_iter().next();
    Ok(first.map(|e| (e.command, e.location.unwrap_or("".into()))))
}