use super::schema::actions;

#[derive(Insertable, Debug)]
#[table_name="actions"]
pub struct NewAction<'a> {
    pub executed: &'a str,
    pub kind: &'a str,
    pub command: &'a str,
    pub location: Option<&'a str>,
    pub epic: Option<&'a str>,
}

#[derive(Queryable)]
pub struct Action {
    pub id: Option<i32>,
    pub executed: String,
    pub kind: String,
    pub command: String,
    pub location: Option<String>,
    pub epic: Option<String>,
    pub sent: Option<bool>,
}

