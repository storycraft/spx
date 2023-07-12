use spx::FileMap;

#[test]
fn map() {
    let map = FileMap::new();

    let _a = map.get("asdafafs.txt");
}
