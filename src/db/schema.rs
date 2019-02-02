table! {
    attribution (page_id, user) {
        page_id -> Text,
        user -> Text,
        kind -> Text,
    }
}

table! {
    memo (channel, user) {
        channel -> Text,
        user -> Text,
        message -> Text,
    }
}

table! {
    namegen (kind, name) {
        kind -> Char,
        name -> Text,
        frequency -> Int4,
    }
}

table! {
    page (id) {
        id -> Text,
        created_at -> Timestamptz,
        created_by -> Text,
        rating -> Int4,
        title -> Text,
        updated -> Timestamp,
    }
}

table! {
    reminder (id) {
        id -> Int4,
        user -> Text,
        when -> Timestamp,
        message -> Text,
    }
}

table! {
    seen (channel, user) {
        channel -> Text,
        user -> Text,
        first -> Text,
        first_time -> Timestamp,
        latest -> Text,
        latest_time -> Timestamp,
        total -> Int4,
    }
}

table! {
    silence (channel, command) {
        channel -> Text,
        command -> Text,
    }
}

table! {
    tag (page_id, name) {
        page_id -> Text,
        name -> Text,
        updated -> Timestamp,
    }
}

table! {
    tell (id) {
        id -> Int4,
        target -> Text,
        sender -> Text,
        time -> Timestamp,
        message -> Text,
    }
}

table! {
    timer (name) {
        name -> Text,
        minutes -> Int4,
    }
}

joinable!(tag -> page (page_id));

allow_tables_to_appear_in_same_query!(
    attribution,
    memo,
    namegen,
    page,
    reminder,
    seen,
    silence,
    tag,
    tell,
    timer,
);
