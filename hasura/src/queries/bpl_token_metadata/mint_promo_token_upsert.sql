INSERT INTO mint_promo_token (
    signature,
    payer,
    token_owner,
    mint,
    authority,
    promo,
    admin_settings,
    token_account,
    slot
)
    VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9)
ON CONFLICT ON CONSTRAINT mint_promo_token_pkey DO UPDATE 
    SET
        payer = EXCLUDED.payer,
        token_owner = EXCLUDED.token_owner,
        mint = EXCLUDED.mint,
        authority = EXCLUDED.authority,
        promo = EXCLUDED.promo,
        admin_settings = EXCLUDED.admin_settings,
        token_account = EXCLUDED.token_account,
        slot = EXCLUDED.slot,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > mint_promo_token.slot
RETURNING created_at = modified_at