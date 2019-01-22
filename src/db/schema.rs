table! {
    memo (channel, user) {
        channel -> Varchar,
        user -> Varchar,
        message -> Varchar,
    }
}

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
    page (fullname) {
        fullname -> Varchar,
        created_at -> Timestamptz,
        created_by -> Varchar,
        rating -> Int4,
        title -> Varchar,
    }
}

table! {
    reminder (id) {
        id -> Int4,
        user -> Varchar,
        when -> Timestamp,
        message -> Varchar,
    }
}

table! {
    seen (channel, user) {
        channel -> Varchar,
        user -> Varchar,
        first -> Varchar,
        first_time -> Timestamp,
        latest -> Varchar,
        latest_time -> Timestamp,
        total -> Int4,
    }
}

table! {
    silence (channel, command) {
        channel -> Varchar,
        command -> Varchar,
    }
}

table! {
    tag (name, page) {
        name -> Varchar,
        page -> Varchar,
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
    memo,
    name_female,
    name_last,
    name_male,
    page,
    reminder,
    seen,
    silence,
    tag,
    tell,
    user,
);
