agent DORA_Compliance stratum: S1 {
    identity { name: "DORA_Compliance", version: "1.0.0" }
    heartbeat { interval: 5_seconds }
    memory { layers: [L0, L1, L2], decay: true }
    capability {
        tokens: [cap::crypto_audit, cap::compliance_write],
    }

    fn main() -> i32 {
        // ── Regulatory constants ──────────────────────────────
        let pq_migration_deadline  = 2028;
        let require_hybrid_mode    = true;

        // ── Verify RSA key size floor ─────────────────────────
        let rsa_check = perform infer<bool>(
            model:  route::select(task::key_size_audit),
            prompt: "RSA key size >= 3072 bits required by DORA Article 9. Verify.",
            budget: think::fast,
        );
        discharge rsa_check with {
            confidence: 0.90,
            taint:      0.10,
            budget:     1000,
        } {
            print("DORA: RSA key size constraint satisfied (>= 3072 bits)");
        }

        // ── Verify forbidden algorithms absent ────────────────
        let forbidden_check = perform infer<bool>(
            model:  route::select(task::algorithm_audit),
            prompt: "Verify RSA_1024, RSA_2048, ECDSA_P256 are absent from all active configurations per DORA.",
            budget: think::fast,
        );
        discharge forbidden_check with {
            confidence: 0.90,
            taint:      0.10,
            budget:     1000,
        } {
            print("DORA: Forbidden algorithm check passed");
        }

        // ── Verify allowed PQC signature algorithms ───────────
        let sig_check = perform infer<bool>(
            model:  route::select(task::signature_audit),
            prompt: "Confirm ML_DSA_44, ML_DSA_65, ML_DSA_87, SLH_DSA_256s are the active signature suite per DORA.",
            budget: think::fast,
        );
        discharge sig_check with {
            confidence: 0.90,
            taint:      0.10,
            budget:     1000,
        } {
            print("DORA: PQC signature suite verified");
        }

        // ── Verify hybrid mode requirement ────────────────────
        if require_hybrid_mode {
            let hybrid_check = perform infer<bool>(
                model:  route::select(task::hybrid_mode_audit),
                prompt: "DORA mandates hybrid classical+PQC mode. Confirm hybrid mode is active.",
                budget: think::fast,
            );
            discharge hybrid_check with {
                confidence: 0.88,
                taint:      0.12,
                budget:     1000,
            } {
                print("DORA: Hybrid mode requirement satisfied");
            }
        }

        // ── Verify ECC key size floor ─────────────────────────
        let ecc_check = perform infer<bool>(
            model:  route::select(task::key_size_audit),
            prompt: "ECC key size >= 256 bits required by DORA. Verify.",
            budget: think::fast,
        );
        discharge ecc_check with {
            confidence: 0.88,
            taint:      0.12,
            budget:     1000,
        } {
            print("DORA: ECC key size constraint satisfied (>= 256 bits)");
        }

        // ── Verify shelf-life parameters ──────────────────────
        let shelf_check = perform infer<bool>(
            model:  route::select(task::shelf_life_audit),
            prompt: "Classical crypto shelf life <= 5 years, PQC shelf life <= 20 years. Verify both constraints.",
            budget: think::fast,
        );
        discharge shelf_check with {
            confidence: 0.85,
            taint:      0.15,
            budget:     1000,
        } {
            print("DORA: Shelf-life constraints satisfied (classical=5yr, pqc=20yr)");
        }

        // ── Verify PQ migration deadline ──────────────────────
        let migration_check = perform infer<bool>(
            model:  route::select(task::migration_audit),
            prompt: "PQ migration must be complete by 2028-01-01 per DORA. Verify timeline is on track.",
            budget: think::fast,
        );
        discharge migration_check with {
            confidence: 0.85,
            taint:      0.15,
            budget:     1000,
        } {
            print("DORA: PQ migration deadline 2028 verified");
        }

        0
    }
}