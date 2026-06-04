agent dora_compliance stratum: S1 {
    fn main() -> i32 {
        let min_rsa_key_size = 3072;
        let allowed_signatures = ["ML_DSA_44", "ML_DSA_65", "ML_DSA_87", "SLH_DSA_256s"];
        let forbidden_algorithms = ["RSA_1024", "RSA_2048", "ECDSA_P256"];
        let pq_migration_deadline = 2028;
        let require_hybrid_mode = true;
        let min_ecc_key_size = 256;
        let shelf_life_classical = 5;
        let shelf_life_pqc = 20;

        let result = perform infer<compliance>(
            model: route::select(task::compliance),
            prompt: "DORA_Article_12_Crypto_Agility",
            budget: think::fast,
        );

        discharge result with {
            confidence: 0.90,
            taint: 0.10,
            budget: 1000,
            capability: cap::compliance,
        } {
            print("DORA compliance verified");
        }

        0
    }
}
