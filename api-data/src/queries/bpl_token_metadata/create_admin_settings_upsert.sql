INSERT INTO create_admin_settings (
    signature,
    payer,
    admin_settings,
    slot
)
    VALUES($1, $2, $3, $4)
ON CONFLICT ON CONSTRAINT create_admin_settings_pkey DO UPDATE 
    SET
        payer = EXCLUDED.payer,
        admin_settings = EXCLUDED.admin_settings,
        slot = EXCLUDED.slot,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > create_admin_settings.slot
RETURNING created_at = modified_at