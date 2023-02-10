INSERT INTO device (
    id,
    owner,
    location,
    name,
    uri,
    metadata_json,
    active,
    slot,
    write_version
)
    VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9)
ON CONFLICT ON CONSTRAINT device_pkey DO UPDATE 
    SET
        owner = EXCLUDED.owner,
        location = EXCLUDED.location,
        name = EXCLUDED.name,
        uri =  EXCLUDED.uri,
        metadata_json =  EXCLUDED.metadata_json,
        active = EXCLUDED.active,
        slot = EXCLUDED.slot,
        write_version = EXCLUDED.write_version,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > device.slot
        OR (
            EXCLUDED.slot = device.slot
            AND EXCLUDED.write_version > device.write_version
        )
RETURNING created_at = modified_at