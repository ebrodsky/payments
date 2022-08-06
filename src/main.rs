mod engine;

fn main() {
    let mut engine = engine::PaymentEngine::new();
    let _res = engine.read_csv();
    //engine.print_transactions();
    //engine.print_entries();
    let _res = engine.process_transactions();
    engine.output_accounts();
}