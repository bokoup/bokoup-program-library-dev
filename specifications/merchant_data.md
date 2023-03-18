# Merchant Data

## Version: 0.0.1

## Background

In order to facilitate the placement, issuance and redemption of promotional offers by merchants
with multiple physical locations, each with potentially multiple point of sale devices, bokoup has
established the following basic data structures:

1. Merchant
2. Location
3. Device
4. Campaign

## Merchant

The Merchant account address is a program derived address with the account owner as the seed. It is
designed to allow only one merchant account per owner address.

The on chain account has the following properties:

```rust
pub struct Merchant {
    pub owner: Pubkey,
    pub name: String,
    pub uri: String,
    pub active: bool,
}
```

The uri value refers to a uri where a json file can be loaded. In practice, bokoup uses Arweave for
permanent storage of this json file. The json file is expected to have the following properties:

```json
{
  "name": "String",
  "description": "String",
  "image": "URL -> String ",
  "website": "URL -> String ",
  "attributes": [
    {
      "trait_type": "String",
      "value": "String | Integer"
    }
  ]
}
```

The attributes can be used by merchants or application developers to include additional data fields.

TODO:

1. Recommended image specifications
2. Include bannser / social media image

## Location

The Location account address is a program derived address with merchant address and location name as
seeds. It is designed to allow multiple locations per merchant as long as the names are unique.

The on chain account has the following properties:

```rust
pub struct Location {
    pub merchant: Pubkey,
    pub name: String,
    pub uri: String,
    pub active: bool,
}
```

The uri value refers to a uri where a json file can be loaded. In practice, bokoup uses Arweave for
permanent storage of this json file. The json file is expected to have the following properties:

```json
{
  "name": "String",
  "description": "String",
  "image": "URL -> String ",
  "address": "URL -> String ",
  "attributes": [
    {
      "trait_type": "String",
      "value": "String | Integer"
    }
  ]
}
```

The address field can be any string representation of an address that is parseable by [] as a valid
address. The attributes can be used by merchants or application developers to include additional
data fields.

## Device

The Device account address is a program derived address with location address and device name as
seeds. It is designed to allow multiple devices per location as long as the names are unique.

The on chain account has the following properties:

```rust
pub struct Device {
    pub owner: Pubkey,
    pub location: Pubkey,
    pub name: String,
    pub uri: String,
    pub active: bool,
}
```

The uri value refers to a uri where a json file can be loaded. In practice, bokoup uses Arweave for
permanent storage of this json file. The json file is expected to have the following properties:

```json
{
  "name": "String",
  "description": "String",
  "attributes": [
    {
      "trait_type": "String",
      "value": "String | Integer"
    }
  ]
}
```

The name field should be the identifying reference of the device in the point of sale systems. The
attributes can be used by merchants or application developers to include additional data fields.

## Campaign

The Campaign account address is a program derived address with merchant address and campaign name as
seeds. It is designed to allow multiple devices per location as long as the names are unique.

The on chain account has the following properties:

```rust
pub struct Campaign {
    pub merchant: Pubkey,
    pub name: String,
    pub uri: String,
    pub locations: Vec<Pubkey>,
    pub active: bool,
}
```

The uri value refers to a uri where a json file can be loaded. In practice, bokoup uses Arweave for
permanent storage of this json file. The json file is expected to have the following properties:

```json
{
  "name": "String",
  "description": "String",
  "attributes": [
    {
      "trait_type": "String",
      "value": "String | Integer"
    }
  ]
}
```

The attributes can be used by merchants or application developers to include additional data fields.
