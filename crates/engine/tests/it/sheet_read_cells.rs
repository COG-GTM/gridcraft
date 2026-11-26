//! `sheet.read` gives each cell what `cell.get` gives it, over gaps, row and column formats,
//! formulas and spills.

use serde_json::{Value, json};

#[test]
fn sheet_read_matches_cell_get_for_every_cell() {
    let mut s = gridcraft_engine::Session::new();
    s.new_workbook();
    s.execute("range.setValues", json!({"range": "A1", "values": [["a", 1.25, null, 0.5], [null, 2, "b", null], [3, null, true, 7]]})).unwrap();
    s.execute("cell.set", json!({"cell": "B4", "input": "=B1*2"})).unwrap();
    s.execute("cell.set", json!({"cell": "D4", "input": "=SEQUENCE(3)"})).unwrap();
    s.execute("home.numberFormat", json!({"range": "B1:B2", "code": "0.0%"})).unwrap();
    s.execute("home.numberFormat", json!({"range": "3:3", "code": "0.000"})).unwrap();
    s.execute("home.numberFormat", json!({"range": "E:E", "code": "#,##0.00"})).unwrap();
    let range = "A1:F7";
    let plain = s.execute("sheet.read", json!({"range": range})).unwrap();
    let shown = s.execute("sheet.read", json!({"range": range, "formatted": true})).unwrap();
    let formulas = s.execute("sheet.read", json!({"range": range, "formulas": true})).unwrap();
    let mut spilled = 0;
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
            spilled += usize::from(!cell["spilledFrom"].is_null());
        }
    }
    assert!(spilled > 0, "the read covers spilled cells");
}
