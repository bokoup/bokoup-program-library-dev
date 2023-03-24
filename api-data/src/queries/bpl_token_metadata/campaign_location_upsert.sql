INSERT INTO campaign_location (
    id,
    campaign,
    location,
    slot,
    write_version
)
    VALUES($1, $2, $3, $4, $5)
ON CONFLICT ON CONSTRAINT campaign_location_pkey DO UPDATE 
    SET
        campaign = EXCLUDED.campaign,
        location = EXCLUDED.location,
        slot = EXCLUDED.slot,
        write_version = EXCLUDED.write_version,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > campaign_location.slot
        OR (
            EXCLUDED.slot = campaign_location.slot
            AND EXCLUDED.write_version > campaign_location.write_version
        )
RETURNING created_at = modified_at