//! Auto Fill Options: redoing the last fill drag as a copy, a series, formats only or values only.

use gridcraft_engine::Session;
use gridcraft_engine::core::RangeRef;
use gridcraft_ui_egui::grid::{FILL_OPTION_MODES, FillOptions, refill};
use serde_json::json;

fn filled() -> (Session, FillOptions) {
    let mut s = Session::new();
    s.new_workbook();
    s.execute("cell.set", json!({"cell": "A1", "input": "1"})).unwrap();
    s.execute("cell.set", json!({"cell": "A2", "input": "2"})).unwrap();
    s.execute("home.bold", json!({"range": "A1:A2"})).unwrap();
    s.execute("edit.autoFill", json!({"source": "A1:A2", "target": "A1:A6", "mode": "series"})).unwrap();
    let undo_len = s.active().unwrap().undo.len();
    let opts =
        FillOptions { source: RangeRef::parse("A1:A2").unwrap(), target: RangeRef::parse("A1:A6").unwrap(), mode: "series", open: false, undo_len };
    (s, opts)
}

fn text(s: &Session, cell: &str) -> String {
    let wb = &s.active().unwrap().wb;
    let sheet = wb.active().unwrap();
    gridcraft_engine::display::value_text(wb, sheet, gridcraft_engine::core::CellRef::parse(cell).unwrap())
}

fn bold(s: &Session, cell: &str) -> bool {
    let wb = &s.active().unwrap().wb;
    let sheet = wb.active().unwrap();
    wb.styles.get(sheet.style_id(gridcraft_engine::core::CellRef::parse(cell).unwrap())).font.bold
}

#[test]
fn copy_cells_repeats_the_source() {
    let (mut s, opts) = filled();
    refill(&mut s, &opts, "copy").unwrap();
    let d = s.active().unwrap();
    assert_eq!(d.selection.current(), opts.target);
    assert_eq!(d.undo.len(), opts.undo_len, "the first fill was replaced, not stacked");
    assert_eq!((text(&s, "A3"), text(&s, "A4")), ("1".into(), "2".into()));
}

#[test]
fn every_menu_mode_is_accepted() {
    for (_, mode) in FILL_OPTION_MODES {
        let (mut s, opts) = filled();
        refill(&mut s, &opts, mode).unwrap();
    }
}

#[test]
fn series_is_what_the_drag_made() {
    let (s, _) = filled();
    assert_eq!((text(&s, "A3"), text(&s, "A6")), ("3".into(), "6".into()));
}

#[test]
fn values_only_and_formats_only_split_the_fill() {
    let (mut s, opts) = filled();
    refill(&mut s, &opts, "values").unwrap();
    assert_eq!(text(&s, "A5"), "5");
    assert!(!bold(&s, "A5"), "values only leaves the formatting behind");
    let (mut s, opts) = filled();
    refill(&mut s, &opts, "formats").unwrap();
    assert!(bold(&s, "A5"));
    assert_eq!(text(&s, "A5"), "");
}
