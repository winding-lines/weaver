use ::errors::*;
use ::store::Store;

use diesel::prelude::*;
use super::models::Action;

pub fn history<T: AsRef<str>>(store: &mut Store, epic:Option<T>) ->  Result<Vec<String>> {
    use super::schema::actions::dsl::*;

    let entries = actions.limit(5).load::<Action>(store.connection())
        .chain_err(|| "fetching actions")?;
    let mut out = Vec::new();

    for entry in entries {
        out.push(entry.command);
    }

    Ok(out)
}