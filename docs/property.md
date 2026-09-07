# Property, Buildings & Construction

Phase 6 adds physical property and building systems to the N simulation.

## Parcels

Land parcels are the base unit of real property. Each parcel belongs to a district and has a zone type:

- **Residential** — houses and apartments
- **Commercial** — shops and offices
- **Industrial** — factories and warehouses

Parcels are created at genesis (4 per district, 20 total). Each parcel has:
- A zone type (determines what buildings are compatible)
- An owner (unowned, company-owned, or agent-owned)
- A current value that appreciates over time
- An optional building occupying it

## Buildings

Buildings occupy parcels and provide capacity:

| Type | Zone | Capacity |
|------|------|----------|
| House | Residential | 2 |
| Apartment | Residential | 4 |
| Shop | Commercial | 3 |
| Office | Commercial | 4 |
| Factory | Industrial | 5 |
| Warehouse | Industrial | 6 |
| Farm | Industrial | 4 |
| Water Plant | Industrial | 3 |

Buildings are created either at genesis (one per company on a compatible parcel) or through construction. Each building tracks:
- Its parcel location
- Owner and operator
- Capacity (agents or production slots)
- Operational status
- Construction state

## Properties

Property records link buildings to ownership. Each property tracks:
- The parcel it sits on
- The building on that parcel
- Ownership (who owns it)
- Current value (derived from parcel value)

## Construction

Companies can expand by constructing new buildings:

1. **Land acquisition** — company selects an unoccupied, zone-compatible parcel
2. **Investment** — company pays money cost from cash reserves
3. **Construction project** — project tracks progress, required labor, and resources
4. **Completion** — building is created, property record is established

### Construction Recipes

Each building type has a recipe defining:
- Money cost
- Labor hours required
- Duration (ticks to complete)
- Capacity of the resulting building

### Construction Flow

```
Company has profit > expansion threshold
→ checks cooldown since last expansion
→ finds suitable unoccupied parcel
→ creates ConstructionProject
→ each tick: labor progress applied
→ when progress >= duration: building activated
→ property record created
→ company building_ids updated
→ district capacity metrics recalculated
```

## Housing

Residential buildings provide housing for agents:
- Agents are assigned to available residential buildings
- Assignment respects building capacity
- Agents lose housing if building is destroyed
- Housing affects district resident counts

## District Metrics

Each district tracks:
- Parcel IDs
- Residential, commercial, and industrial capacity
- Number of residents
- Total property value
- Active and completed construction counts

## Economic Causality

```
Company profit → expansion demand → land acquisition
→ construction investment → resource+labor consumption
→ building completion → capacity increase → economic output
```

## Tests

20 tests covering:
- Parcel creation and district assignment
- Zoning and building compatibility
- Building occupancy and capacity
- Property ownership consistency
- Construction recipe validation
- Manual construction flow
- Money conservation with construction
- Housing assignment and occupancy bounds
- District metrics tracking
- Property value appreciation
- Deterministic replay

## Files

- `property.rs` — Parcel, Property, Building, ConstructionProject structs
- `lib.rs` — Genesis, tick processing, test suite
