fn main() {
    cynic_codegen::register_schema("backend-schema")
        .from_sdl_file("../../libs/generated/backend-schema.graphql")
        .unwrap()
        .as_default()
        .unwrap();
}
