INSERT INTO admin_settings (
    id,
    platform,
    create_promo_lamports,
    burn_promo_token_lamports,
    slot,
    write_version
)
    VALUES($1, $2, $3, $4, $5, $6)
ON CONFLICT ON CONSTRAINT admin_settings_pkey DO UPDATE 
    SET
        platform = EXCLUDED.platform,
        create_promo_lamports = EXCLUDED.create_promo_lamports,
        burn_promo_token_lamports = EXCLUDED.burn_promo_token_lamports,
        slot = EXCLUDED.slot,
        write_version = EXCLUDED.write_version,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > admin_settings.slot
        OR (
            EXCLUDED.slot = admin_settings.slot
            AND EXCLUDED.write_version > admin_settings.write_version
        )
RETURNING created_at = modified_at