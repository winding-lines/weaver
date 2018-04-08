table! {
    actions2 (id) {
        id -> Nullable<Integer>,
        command_id -> Nullable<Integer>,
        executed -> Text,
        location_id -> Nullable<Integer>,
        epic_id -> Nullable<Integer>,
        sent -> Nullable<Bool>,
        annotation -> Nullable<Text>,
    }
}

table! {
    commands (id) {
        id -> Nullable<Integer>,
        kind -> Text,
        command -> Text,
    }
}

table! {
    epics (id) {
        id -> Nullable<Integer>,
        name -> Text,
    }
}

table! {
    locations (id) {
        id -> Nullable<Integer>,
        location -> Text,
    }
}

joinable!(actions2 -> commands (command_id));
joinable!(actions2 -> epics (epic_id));
joinable!(actions2 -> locations (location_id));

allow_tables_to_appear_in_same_query!(
    actions2,
    commands,
    epics,
    locations,
);
