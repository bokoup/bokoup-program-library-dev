INSERT INTO create_device (
    signature,
    payer,
    merchant,
    location,
    device,
    memo,
    slot
)
    VALUES($1, $2, $3, $4, $5, $6, $7)
ON CONFLICT ON CONSTRAINT create_device_pkey DO UPDATE 
    SET
        payer = EXCLUDED.payer,
        merchant = EXCLUDED.merchant,
        location = EXCLUDED.location,
        device = EXCLUDED.device,
        memo = EXCLUDED.memo,
        slot = EXCLUDED.slot,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > create_device.slot
RETURNING created_at = modified_at