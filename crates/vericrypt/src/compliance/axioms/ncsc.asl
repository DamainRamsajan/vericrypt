agent NCSC_Compliance stratum: S1 {
    identity { name: "NCSC_Compliance", version: "1.0.0" }
    heartbeat { interval: 5_seconds }
    memory { layers: [L0, L1, L2], decay: true }
    capability {
        tokens: [cap::crypto_audit, cap::compliance_write],
    }

    fn main() -> i32 {
        // ── Regulatory constants ──────────────────────────────
        let min_rsa_key_size      = 3072;
        let pq_migration_deadline = 2030;
        let require_hybrid_mode   = false;
        let phase1_discovery      = true;
        let phase2_planning       = true;
        let phase3_execution      = true;

        // ── Verify RSA key size floor ─────────────────────────
        let rsa_check = perform infer<bool>(
            model:  route::select(task::key_size_audit),
            prompt: "RSA key size >= 3072 bits required by NCSC PQC guidance. Verify.",
            budget: think::fast,
        );
        discharge rsa_check with {
            confidence: 0.90,
            taint:      0.10,
            budget:     1000,
        } {
            print("NCSC: RSA key size constraint satisfied (>= 3072 bits)");
        }

        // ── Verify forbidden algorithms absent ────────────────
        let forbidden_check = perform infer<bool>(
            model:  route::select(task::algorithm_audit),
            prompt: "Verify RSA_1024, RSA_2048, ECDSA_P256 are absent from all active configurations per NCSC guidance.",
            budget: think::fast,
        );
        discharge forbidden_check with {
            confidence: 0.90,
            taint:      0.10,
            budget:     1000,
        } {
            print("NCSC: Forbidden algorithm check passed");
        }

        // ── Verify allowed PQC signature algorithms ───────────
        let sig_check = perform infer<bool>(
            model:  route::select(task::signature_audit),
            prompt: "Confirm ML_DSA_44, ML_DSA_65, ML_DSA_87, SLH_DSA_256s are the active signature suite per NCSC.",
            budget: think::fast,
        );
        discharge sig_check with {
            confidence: 0.90,
            taint:      0.10,
            budget:     1000,
        } {
            print("NCSC: PQC signature suite verified");
        }

        // ── Hybrid mode: not required by NCSC ────────────────
        if !require_hybrid_mode {
            print("NCSC: Hybrid mode not mandated; classical-or-PQC permitted");
        }

        // ── Phase 1: Discovery ────────────────────────────────
        if phase1_discovery {
            let phase1_check = perform infer<bool>(
                model:  route::select(task::discovery_audit),
                prompt: "NCSC Phase 1 requires complete discovery of all cryptographic assets. Confirm Phase 1 is complete.",
                budget: think::fast,
            );
            discharge phase1_check with {
                confidence: 0.88,
                taint:      0.12,
                budget:     1000,
            } {
                print("NCSC: Phase 1 Discovery complete");
            }
        }

        // ── Phase 2: Planning ─────────────────────────────────
        if phase2_planning {
            let phase2_check = perform infer<bool>(
                model:  route::select(task::planning_audit),
                prompt: "NCSC Phase 2 requires a documented PQC migration plan. Confirm Phase 2 plan exists and is approved.",
                budget: think::fast,
            );
            discharge phase2_check with {
                confidence: 0.88,
                taint:      0.12,
                budget:     1000,
            } {
                print("NCSC: Phase 2 Planning complete");
            }
        }

        // ── Phase 3: Execution ────────────────────────────────
        if phase3_execution {
            let phase3_check = perform infer<bool>(
                model:  route::select(task::execution_audit),
                prompt: "NCSC Phase 3 requires active execution of PQC migration. Confirm Phase 3 is in progress or complete.",
                budget: think::fast,
            );
            discharge phase3_check with {
                confidence: 0.85,
                taint:      0.15,
                budget:     1000,
            } {
                print("NCSC: Phase 3 Execution verified");
            }
        }

        // ── Verify PQ migration deadline ──────────────────────
        let migration_check = perform infer<bool>(
            model:  route::select(task::migration_audit),
            prompt: "PQ migration must be complete by 2030-01-01 per NCSC guidance. Verify timeline is on track.",
            budget: think::fast,
        );
        discharge migration_check with {
            confidence: 0.85,
            taint:      0.15,
            budget:     1000,
        } {
            print("NCSC: PQ migration deadline 2030 verified");
        }

        0
    }
}