INSERT INTO merchant (
    id,
    owner,
    name,
    uri,
    metadata_json,
    active,
    slot,
    write_version
)
    VALUES($1, $2, $3, $4, $5, $6, $7, $8)
ON CONFLICT ON CONSTRAINT merchant_pkey DO UPDATE 
    SET
        owner = EXCLUDED.owner,
        name = EXCLUDED.name,
        uri =  EXCLUDED.uri,
        metadata_json =  EXCLUDED.metadata_json,
        active = EXCLUDED.active,
        slot = EXCLUDED.slot,
        write_version = EXCLUDED.write_version,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > merchant.slot
        OR (
            EXCLUDED.slot = merchant.slot
            AND EXCLUDED.write_version > merchant.write_version
        )
RETURNING created_at = modified_at