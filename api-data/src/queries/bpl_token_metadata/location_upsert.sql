INSERT INTO location (
    id,
    merchant,
    name,
    uri,
    metadata_json,
    active,
    slot,
    write_version
)
    VALUES($1, $2, $3, $4, $5, $6, $7, $8)
ON CONFLICT ON CONSTRAINT location_pkey DO UPDATE 
    SET
        merchant = EXCLUDED.merchant,
        name = EXCLUDED.name,
        uri =  EXCLUDED.uri,
        metadata_json =  EXCLUDED.metadata_json,
        active = EXCLUDED.active,
        slot = EXCLUDED.slot,
        write_version = EXCLUDED.write_version,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > location.slot
        OR (
            EXCLUDED.slot = location.slot
            AND EXCLUDED.write_version > location.write_version
        )
RETURNING created_at = modified_at