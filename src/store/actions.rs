use ::display::FormattedAction;
use ::errors::*;
use ::store::Store;
use diesel::prelude::*;
use super::backends::models::Action;

pub fn history<T: AsRef<str>>(store: &mut Store, _epic: &Option<T>) -> Result<Vec<FormattedAction>> {
    use super::backends::schema::actions::dsl::*;

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
        });
    }

    Ok(out)
}