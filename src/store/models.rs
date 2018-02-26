use super::schema::actions;

#[derive(Insertable, Debug)]
#[table_name="actions"]
pub struct NewAction<'a> {
    pub executed: &'a str,
    pub kind: &'a str,
    pub command: &'a str,
    pub location: Option<&'a str>
}

