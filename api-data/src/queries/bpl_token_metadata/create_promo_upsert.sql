INSERT INTO create_promo (
    signature,
    payer,
    payer_balance,
    owner,
    owner_balance,
    merchant,
    campaign,
    campaign_balance,
    mint,
    metadata,
    authority,
    promo,
    platform,
    platform_balance,
    admin_settings,
    memo,
    slot
)
    VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
ON CONFLICT ON CONSTRAINT create_promo_pkey DO UPDATE 
    SET
        payer = EXCLUDED.payer,
        payer_balance = EXCLUDED.payer_balance,
        owner = EXCLUDED.owner,
        owner_balance = EXCLUDED.owner_balance,
        merchant = EXCLUDED.merchant,
        campaign = EXCLUDED.campaign,
        campaign_balance = EXCLUDED.campaign_balance,
        mint = EXCLUDED.mint,
        metadata = EXCLUDED.metadata,
        authority = EXCLUDED.authority,
        promo = EXCLUDED.promo,
        platform = EXCLUDED.platform,
        platform_balance = EXCLUDED.platform_balance,
        admin_settings = EXCLUDED.admin_settings,
        memo = EXCLUDED.memo,
        slot = EXCLUDED.slot,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > create_promo.slot
RETURNING created_at = modified_at