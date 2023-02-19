INSERT INTO create_merchant (
    signature,
    payer,
    merchant,
    memo,
    slot
)
    VALUES($1, $2, $3, $4, $5)
ON CONFLICT ON CONSTRAINT create_merchant_pkey DO UPDATE 
    SET
        payer = EXCLUDED.payer,
        merchant = EXCLUDED.merchant,
        memo = EXCLUDED.memo,
        slot = EXCLUDED.slot,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > create_merchant.slot
RETURNING created_at = modified_at