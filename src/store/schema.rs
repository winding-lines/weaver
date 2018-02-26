table! {
    actions (id) {
        id -> Nullable<Integer>,
        executed -> Text,
        kind -> Text,
        command -> Text,
        location -> Nullable<Text>,
    }
}
