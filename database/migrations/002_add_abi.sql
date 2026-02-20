ALTER TABLE contracts ADD COLUMN abi JSONB;

CREATE TABLE contract_abis (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    contract_id UUID NOT NULL REFERENCES contracts(id) ON DELETE CASCADE,
    version VARCHAR(50) NOT NULL,
    abi JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(contract_id, version)
);

CREATE INDEX idx_contract_abis_contract_id ON contract_abis(contract_id);
