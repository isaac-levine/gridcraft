//! A value typed into a merged area that holds an array formula survives an XLSX save (#3397).

use gridcraft_engine::Session;
use gridcraft_engine::core::{CellRef, Value};
use serde_json::json;

fn reopen(session: &mut Session) -> Session {
    let saved = session.execute("file.saveBytes", json!({"format": "xlsx"})).unwrap();
    let mut reopened = Session::new();
    reopened.execute("file.open", json!({"name": "merged.xlsx", "base64": saved["base64"]})).unwrap();
    reopened
}

fn value(session: &Session, a1: &str) -> Value {
    let at = CellRef::parse(a1).unwrap();
    session.doc().unwrap().wb.active().unwrap().cell(at).map(|c| c.value.clone()).unwrap_or(Value::Empty)
}

fn merged(array: bool) -> Session {
    let mut s = Session::new();
    s.new_workbook();
    s.execute("selection.set", json!({"range": "J9:M14"})).unwrap();
    s.execute("home.mergeCells", json!({})).unwrap();
    if array {
        s.execute("cell.set", json!({"array": true, "cell": "K11", "input": "=1+1"})).unwrap();
    }
    let typed = s.execute("cell.set", json!({"cell": "L9", "input": "-5.89"}));
    // Excel refuses an edit to part of an array, merged or not.
    assert_eq!(typed.is_err(), array, "array={array}");
    s
}

#[test]
fn value_in_merged_area_survives_save_and_array_cells_are_refused() {
    for array in [false, true] {
        let mut s = merged(array);
        let before = value(&s, "L9");
        let back = reopen(&mut s);
        assert_eq!(value(&back, "L9"), before, "array={array}");
    }
}
