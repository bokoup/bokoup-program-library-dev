INSERT INTO mint_promo_token (
    signature,
    payer,
    device_owner,
    device,
    campaign,
    campaign_location,
    token_owner,
    mint,
    authority,
    promo,
    token_account,
    memo,
    slot
)
    VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
ON CONFLICT ON CONSTRAINT mint_promo_token_pkey DO UPDATE 
    SET
        payer = EXCLUDED.payer,
        device_owner = EXCLUDED.device_owner,
        device = EXCLUDED.device,
        campaign = EXCLUDED.campaign,
        campaign_location = EXCLUDED.campaign_location,
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