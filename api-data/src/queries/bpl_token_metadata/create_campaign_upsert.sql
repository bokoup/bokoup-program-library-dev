INSERT INTO create_campaign (
    signature,
    payer,
    merchant,
    campaign,
    lamports,
    memo,
    slot
)
    VALUES($1, $2, $3, $4, $5, $6, $7)
ON CONFLICT ON CONSTRAINT create_campaign_pkey DO UPDATE 
    SET
        payer = EXCLUDED.payer,
        merchant = EXCLUDED.merchant,
        campaign = EXCLUDED.campaign,
        lamports = EXCLUDED.lamports,
        memo = EXCLUDED.memo,
        slot = EXCLUDED.slot,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > create_campaign.slot
RETURNING created_at = modified_at