INSERT INTO mint_promo_token (
    signature,
    payer,
    location,
    device,
    campaign,
    token_owner,
    mint,
    authority,
    promo,
    token_account,
    memo,
    slot
)
    VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
ON CONFLICT ON CONSTRAINT mint_promo_token_pkey DO UPDATE 
    SET
        payer = EXCLUDED.payer,
        location = EXCLUDED.location,
        device = EXCLUDED.device,
        campaign = EXCLUDED.campaign,
        token_owner = EXCLUDED.token_owner,
        mint = EXCLUDED.mint,
        authority = EXCLUDED.authority,
        promo = EXCLUDED.promo,
        token_account = EXCLUDED.token_account,
        memo = EXCLUDED.memo,
        slot = EXCLUDED.slot,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > mint_promo_token.slot
RETURNING created_at = modified_at