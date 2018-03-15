table! {
    actions (id) {
        id -> Nullable<Integer>,
        executed -> Text,
        kind -> Text,
        command -> Text,
        location -> Nullable<Text>,
        epic -> Nullable<Text>,
        sent -> Nullable<Bool>,
        annotation -> Nullable<Text>,
        tags -> Nullable<Text>,
    }
}
