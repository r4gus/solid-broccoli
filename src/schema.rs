table! {
    rms (id) {
        id -> Int4,
        reps -> Int4,
        exercise -> Varchar,
        weight -> Float8,
        unit -> Varchar,
        lifted -> Timestamp,
        uid -> Int4,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        email -> Varchar,
        password_hash -> Varchar,
        first_name -> Varchar,
        last_name -> Varchar,
        street -> Varchar,
        house_number -> Varchar,
        zip -> Varchar,
        city -> Varchar,
        phone -> Varchar,
        img_path -> Varchar,
        is_admin -> Bool,
        verified -> Bool,
    }
}

joinable!(rms -> users (uid));

allow_tables_to_appear_in_same_query!(
    rms,
    users,
);
