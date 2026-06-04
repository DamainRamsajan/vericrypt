agent ncsc_compliance stratum: S1 {
    fn main() -> i32 {
        let min_rsa_key_size = 3072;
        let allowed_signatures = ["ML_DSA_44", "ML_DSA_65", "ML_DSA_87", "SLH_DSA_256s"];
        let forbidden_algorithms = ["RSA_1024", "RSA_2048", "ECDSA_P256"];
        let pq_migration_deadline = 2030;
        let require_hybrid_mode = false;
        let phase1_discovery = true;
        let phase2_planning = true;
        let phase3_execution = true;

        let result = perform infer<compliance>(
            model: route::select(task::compliance),
            prompt: "NCSC_Phase_Compliance",
            budget: think::fast,
        );

        discharge result with {
            confidence: 0.85,
            taint: 0.15,
            budget: 1000,
            capability: cap::compliance,
        } {
            print("NCSC compliance verified");
        }

        0
    }
}
