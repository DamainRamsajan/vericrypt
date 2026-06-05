fn main() -> i32 {
    let rsa_floor = 3072;
    let hybrid_required = 1;
    let migration_deadline = 2028;
    let pqc_sig_suite = 4;
    rsa_floor + hybrid_required + pqc_sig_suite
}
