table! {
    ban (id) {
        id -> Int4,
        nicks -> Varchar,
        ips -> Varchar,
        reason -> Varchar,
        expiry -> Date,
    }
}

table! {
    page (url) {
        url -> Varchar,
        name -> Varchar,
        author -> Varchar,
        votes -> Int4,
    }
}

table! {
    property (key) {
        key -> Varchar,
        value -> Varchar,
    }
}

table! {
    reminder (id) {
        id -> Int4,
        nick -> Varchar,
        when -> Timestamp,
        message -> Varchar,
    }
}

table! {
    seen (nick) {
        nick -> Varchar,
        first -> Varchar,
        first_time -> Timestamp,
        latest -> Varchar,
        latest_time -> Timestamp,
        total -> Int4,
    }
}

table! {
    silence (id) {
        id -> Int4,
        command -> Varchar,
        channel -> Varchar,
    }
}

table! {
    tell (id) {
        id -> Int4,
        target -> Varchar,
        sender -> Varchar,
        time -> Timestamp,
        message -> Varchar,
    }
}

table! {
    user (nick) {
        nick -> Varchar,
        auth -> Int4,
        pronouns -> Nullable<Varchar>,
    }
}

allow_tables_to_appear_in_same_query!(
    ban,
    page,
    property,
    reminder,
    seen,
    silence,
    tell,
    user,
);
