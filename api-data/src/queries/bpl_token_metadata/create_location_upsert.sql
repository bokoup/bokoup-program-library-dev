INSERT INTO create_location (
    signature,
    payer,
    merchant,
    location,
    memo,
    slot
)
    VALUES($1, $2, $3, $4, $5, $6)
ON CONFLICT ON CONSTRAINT create_location_pkey DO UPDATE 
    SET
        payer = EXCLUDED.payer,
        merchant = EXCLUDED.merchant,
        location = EXCLUDED.location,
        memo = EXCLUDED.memo,
        slot = EXCLUDED.slot,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > create_location.slot
RETURNING created_at = modified_at