use rubik_solver::macro_solver::L2CTable;

fn main() {
    println!("Generating table...");
    let table = L2CTable::generate_for_6x6();
    println!("Table size: {}", table.table.len());
}
