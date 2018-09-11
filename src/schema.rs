table! {
    actions2 (id) {
        id -> Nullable<Integer>,
        command_id -> Nullable<Integer>,
        executed -> Text,
        location_id -> Nullable<Integer>,
        epic_id -> Nullable<Integer>,
        sent -> Nullable<Bool>,
        annotation -> Nullable<Text>,
        host_id -> Nullable<Integer>,
        status -> Nullable<Integer>,
    }
}

table! {
    commands (id) {
        id -> Nullable<Integer>,
        kind -> Text,
        command -> Text,
        page_id -> Nullable<Integer>,
    }
}

table! {
    epics (id) {
        id -> Nullable<Integer>,
        name -> Text,
    }
}

table! {
    hosts (id) {
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

table! {
    pages (id) {
        id -> Nullable<Integer>,
        normalized_url -> Text,
        title -> Nullable<Text>,
    }
}

table! {
    url_restrictions (id) {
        id -> Nullable<Integer>,
        kind -> Text,
        url_expr -> Text,
        title_match -> Nullable<Text>,
        body_match -> Nullable<Text>,
    }
}

joinable!(actions2 -> commands (command_id));
joinable!(actions2 -> epics (epic_id));
joinable!(actions2 -> hosts (host_id));
joinable!(actions2 -> locations (location_id));
joinable!(commands -> pages (page_id));

allow_tables_to_appear_in_same_query!(
    actions2,
    commands,
    epics,
    hosts,
    locations,
    pages,
    url_restrictions,
);
