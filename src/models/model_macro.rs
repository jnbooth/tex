macro_rules! model {
    ( $struct_name:ident ; $db_struct_name:ident ; $table_name:expr ; {
        $( $attr_name:ident : $attr_type:ty ),*
    }) => {
        #[derive(Queryable)]
        pub struct $db_struct_name {
            pub id: i32,
            $( pub $attr_name : $attr_type ),*
        }

        #[table_name=$table_name]
        #[derive(Insertable)]
        pub struct $struct_name {
            $( pub $attr_name : $attr_type ),*
        }

        impl From<$db_struct_name> for $struct_name {
            fn from(x: $db_struct_name) -> $struct_name {
                $struct_name {
                    $( $attr_name: x.$attr_name ),*
                }
            }
        }
    }
}
