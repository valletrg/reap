use comfy_table::{Table, Cell, Color, ContentArrangement};
fn main() {
    crossterm::style::force_color_output(true);
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["PID", "PORT"]);
    table.add_row(vec![
        Cell::new("53737"),
        Cell::new("6463").fg(Color::Green),
    ]);
    let s = table.to_string();
    println!("Raw table output:");
    println!("{}", s);
    println!("\nHex dump:");
    for b in s.bytes() {
        print!("{:02x} ", b);
    }
    println!();
}
