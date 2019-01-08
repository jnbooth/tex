table! {
    name_female (name) {
        name -> Varchar,
        frequency -> Int4,
        probability -> Float8,
    }
}

table! {
    name_last (name) {
        name -> Varchar,
        frequency -> Int4,
        probability -> Float8,
    }
}

table! {
    name_male (name) {
        name -> Varchar,
        frequency -> Int4,
        probability -> Float8,
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
    reminder (id) {
        id -> Int4,
        nick -> Varchar,
        when -> Timestamp,
        message -> Varchar,
    }
}

table! {
    seen (id) {
        id -> Int4,
        channel -> Varchar,
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
        channel -> Varchar,
        command -> Varchar,
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
    name_female,
    name_last,
    name_male,
    page,
    reminder,
    seen,
    silence,
    tell,
    user,
);
