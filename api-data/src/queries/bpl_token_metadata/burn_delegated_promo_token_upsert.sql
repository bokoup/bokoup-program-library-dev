INSERT INTO burn_delegated_promo_token (
    signature,
    payer,
    payer_balance,
    location,
    device,
    campaign,
    campaign_balance,
    mint,
    authority,
    promo,
    platform,
    platform_balance,
    admin_settings,
    token_account,
    memo,
    slot
)
    VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
ON CONFLICT ON CONSTRAINT burn_delegated_promo_token_pkey DO UPDATE 
    SET
        payer = EXCLUDED.payer,
        payer_balance = EXCLUDED.payer_balance,
        location = EXCLUDED.location,
        device = EXCLUDED.device,
        campaign = EXCLUDED.campaign,
        campaign_balance = EXCLUDED.campaign_balance,
        mint = EXCLUDED.mint,
        authority = EXCLUDED.authority,
        promo = EXCLUDED.promo,
        platform = EXCLUDED.platform,
        platform_balance = EXCLUDED.platform_balance,
        admin_settings = EXCLUDED.admin_settings,
        token_account = EXCLUDED.token_account,
        memo = EXCLUDED.memo,
        slot = EXCLUDED.slot,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > burn_delegated_promo_token.slot
RETURNING created_at = modified_at