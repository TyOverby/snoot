fn foo() {
    parse_simple_ok(
        "() ()",
        vec![SimpleSexpr::List {
                 opening: "(".into(),
                 closing: ")".into(),
                 entire: "()".into(),
                 children: vec![],
             },
             SimpleSexpr::List {
                 opening: "(".into(),
                 closing: ")".into(),
                 entire: "()".into(),
                 children: vec![],
            }]);
}
