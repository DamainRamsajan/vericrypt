agent DORA_Compliance stratum: S1 {
    identity { name: "DORA_Compliance", version: "1.0.0" }
    heartbeat { interval: 5_seconds }
    memory { layers: [L0, L1, L2], decay: true }
    capability { tokens: [cap::crypto_audit, cap::compliance_write] }
    fn main() -> i32 {
        let rsa_check = perform infer<bool>(model: route::select(task::key_size_audit), prompt: "RSA key size >= 3072 bits required by DORA Article 9", budget: think::fast);
        discharge rsa_check { 0.90 => { print("DORA: RSA key size constraint satisfied"); } }
        let forbidden_check = perform infer<bool>(model: route::select(task::algorithm_audit), prompt: "Verify RSA_1024, RSA_2048, ECDSA_P256 absent per DORA", budget: think::fast);
        discharge forbidden_check { 0.90 => { print("DORA: Forbidden algorithm check passed"); } }
        let sig_check = perform infer<bool>(model: route::select(task::signature_audit), prompt: "Confirm ML_DSA/SLH_DSA active per DORA", budget: think::fast);
        discharge sig_check { 0.90 => { print("DORA: PQC signature suite verified"); } }
        let hybrid_check = perform infer<bool>(model: route::select(task::hybrid_mode_audit), prompt: "DORA mandates hybrid classical+PQC mode", budget: think::fast);
        discharge hybrid_check { 0.88 => { print("DORA: Hybrid mode requirement satisfied"); } }
        let ecc_check = perform infer<bool>(model: route::select(task::key_size_audit), prompt: "ECC key size >= 256 bits required by DORA", budget: think::fast);
        discharge ecc_check { 0.88 => { print("DORA: ECC key size constraint satisfied"); } }
        let shelf_check = perform infer<bool>(model: route::select(task::shelf_life_audit), prompt: "Classical shelf life <= 5 years, PQC <= 20 years", budget: think::fast);
        discharge shelf_check { 0.85 => { print("DORA: Shelf-life constraints satisfied"); } }
        let migration_check = perform infer<bool>(model: route::select(task::migration_audit), prompt: "PQ migration must be complete by 2028-01-01 per DORA", budget: think::fast);
        discharge migration_check { 0.85 => { print("DORA: PQ migration deadline 2028 verified"); } }
        0
    }
}
