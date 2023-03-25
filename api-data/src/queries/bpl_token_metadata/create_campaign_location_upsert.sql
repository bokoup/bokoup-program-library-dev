INSERT INTO create_campaign_location (
    signature,
    payer,
    owner,
    merchant,
    campaign,
    location,
    campaign_location,
    memo,
    slot
)
    VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9)
ON CONFLICT ON CONSTRAINT create_campaign_location_pkey DO UPDATE 
    SET
        payer = EXCLUDED.payer,
        owner = EXCLUDED.owner,
        merchant = EXCLUDED.merchant,
        campaign = EXCLUDED.campaign,
        location = EXCLUDED.location,
        memo = EXCLUDED.memo,
        slot = EXCLUDED.slot,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > create_campaign_location.slot
RETURNING created_at = modified_at