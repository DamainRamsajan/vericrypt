-- Customers
CREATE TABLE customers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT UNIQUE NOT NULL,
    organization TEXT NOT NULL,
    contact_name TEXT,
    country TEXT,
    created_at TIMESTAMPTZ DEFAULT now(),
    tier TEXT DEFAULT 'trial' CHECK (tier IN ('trial', 'standard', 'enterprise'))
);

-- Licenses
CREATE TABLE licenses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID REFERENCES customers(id) NOT NULL,
    license_key TEXT UNIQUE NOT NULL,
    binary_hash TEXT NOT NULL,
    issued_at TIMESTAMPTZ DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    revoked BOOLEAN DEFAULT false,
    max_assets INTEGER DEFAULT 100000,
    features JSONB DEFAULT '[]'
);

-- Download audit log
CREATE TABLE downloads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    license_id UUID REFERENCES licenses(id),
    binary_name TEXT NOT NULL,
    platform TEXT NOT NULL,
    downloaded_at TIMESTAMPTZ DEFAULT now(),
    ip_hash TEXT,
    user_agent TEXT
);

-- Binary versions
CREATE TABLE binaries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    platform TEXT NOT NULL,
    storage_path TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    binary_hash TEXT NOT NULL,
    uploaded_at TIMESTAMPTZ DEFAULT now(),
    is_latest BOOLEAN DEFAULT false
);

-- Indexes
CREATE INDEX idx_licenses_customer ON licenses(customer_id);
CREATE INDEX idx_licenses_key ON licenses(license_key);
CREATE INDEX idx_binaries_latest ON binaries(name, platform) WHERE is_latest = true;