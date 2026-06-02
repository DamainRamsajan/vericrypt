agent NIST_Compliance {
    contract {
        min_rsa_key_size: 3072,
        allowed_signatures: [ML_DSA_44, ML_DSA_65, ML_DSA_87, SLH_DSA_256s],
        pq_migration_required: true,
        pq_migration_deadline: 2035-01-01,
        forbidden_algorithms: [RSA_1024, RSA_2048, ECDSA_P256],
        require_hybrid_mode: false,
        crypto_agility_required: true,
    }
}
