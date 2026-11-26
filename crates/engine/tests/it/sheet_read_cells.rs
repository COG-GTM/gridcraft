//! `sheet.read` gives each cell what `cell.get` gives it, over gaps, row and column formats,
//! formulas and spills.

use serde_json::{Value, json};

#[test]
fn sheet_read_matches_cell_get_for_every_cell() {
    let mut s = gridcraft_engine::Session::new();
    s.new_workbook();
    s.execute("range.setValues", json!({"range": "A1", "values": [["a", 1.25, null, 0.5], [null, 2, "b", null], [3, null, true, 7]]})).unwrap();
    s.execute("cell.set", json!({"cell": "B4", "input": "=B1*2"})).unwrap();
    s.execute("cell.set", json!({"cell": "D4", "input": "=SEQUENCE(3,2)"})).unwrap();
    s.execute("home.numberFormat", json!({"range": "B1:B2", "code": "0.0%"})).unwrap();
    s.execute("home.numberFormat", json!({"range": "3:3", "code": "0.000"})).unwrap();
    // Spilled cells are not stored, so these whole-line formats reach them through the row and
    // column styles.
    s.execute("home.numberFormat", json!({"range": "5:5", "code": "0.000"})).unwrap();
    s.execute("home.numberFormat", json!({"range": "E:E", "code": "#,##0.00"})).unwrap();
    let range = "A1:F7";
    let plain = s.execute("sheet.read", json!({"range": range})).unwrap();
    let shown = s.execute("sheet.read", json!({"range": range, "formatted": true})).unwrap();
    let formulas = s.execute("sheet.read", json!({"range": range, "formulas": true})).unwrap();
    let mut spilled = 0;
    let mut inherited = 0;
    for row in 0..7usize {
        for col in 0..6usize {
            let a1 = format!("{}{}", char::from(b'A' + col as u8), row + 1);
            let cell = s.execute("cell.get", json!({"cell": a1})).unwrap();
            assert_eq!(plain["values"][row][col], cell["value"], "{a1} value");
            assert_eq!(shown["values"][row][col], cell["text"], "{a1} text");
            let expect = match &cell["formula"] {
                Value::String(f) => json!(f),
                _ => cell["value"].clone(),
            };
            assert_eq!(formulas["values"][row][col], expect, "{a1} formula");
            if !cell["spilledFrom"].is_null() {
                spilled += 1;
                // The spill holds whole numbers, which General shows without a decimal point.
                let decimals = cell["text"].as_str().is_some_and(|t| t.contains('.'));
                inherited += usize::from(decimals && cell["value"].as_f64().is_some_and(|v| v.fract() == 0.0));
            }
        }
    }
    assert!(spilled > 0, "the read covers spilled cells");
    assert!(inherited > 0, "some spilled cell shows a row or column format");
}

#[test]
fn sheet_read_hides_protected_formulas_under_show_formulas() {
    let mut s = gridcraft_engine::Session::new();
    s.new_workbook();
    s.execute("cell.set", json!({"cell": "A1", "input": "=SEQUENCE(2)*7"})).unwrap();
    s.execute("cell.set", json!({"cell": "B1", "input": "=1+1"})).unwrap();
    s.execute("home.formatCells", json!({"range": "A1", "style": {"protection": {"locked": true, "hidden": true}}})).unwrap();
    s.execute("formulas.showFormulas", json!({"on": true})).unwrap();
    let read = |s: &mut gridcraft_engine::Session| s.execute("sheet.read", json!({"range": "A1:B2", "formatted": true})).unwrap()["values"].clone();
    let open = read(&mut s);
    s.execute("review.protectSheet", json!({})).unwrap();
    let locked = read(&mut s);
    for (row, col) in [(0, 0), (0, 1), (1, 0)] {
        let a1 = format!("{}{}", char::from(b'A' + col as u8), row + 1);
        assert_eq!(locked[row][col], s.execute("cell.get", json!({"cell": a1})).unwrap()["text"], "{a1}");
    }
    assert!(open[0][0].as_str().is_some_and(|t| t.starts_with('=')), "unprotected, Show Formulas shows the formula");
    assert!(locked[0][0].as_str().is_some_and(|t| !t.starts_with('=')), "protection hides it");
    assert!(locked[0][1].as_str().is_some_and(|t| t.starts_with('=')), "an unhidden formula still shows");
}
