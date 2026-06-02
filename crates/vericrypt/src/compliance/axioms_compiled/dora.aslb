agent DORA_Compliance {
    contract {
        min_rsa_key_size: 3072,
        allowed_signatures: [ML_DSA_44, ML_DSA_65, ML_DSA_87, SLH_DSA_256s],
        pq_migration_required: true,
        pq_migration_deadline: 2028-01-01,
        forbidden_algorithms: [RSA_1024, RSA_2048, ECDSA_P256],
        require_hybrid_mode: true,
        min_ecc_key_size: 256,
        shelf_life_classical: 5,
        shelf_life_pqc: 20,
    }

    charter {
        dynamic_risk {
            taint_ceiling: 0.20,
        }
    }
}
