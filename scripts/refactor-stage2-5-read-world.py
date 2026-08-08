from pathlib import Path

path = Path("crates/simulation-save/src/binary.rs")
text = path.read_text()

if "fn read_entity_world(cursor:" in text:
    raise SystemExit(0)

old = '''    let human_group_next_id = cursor.read_u64()?;
    let human_group_count = cursor.read_count("human_groups")?;
    let mut human_groups = Vec::with_capacity(human_group_count);
    for _ in 0..human_group_count {
        human_groups.push(HumanGroupEntity {
            id: HumanGroupId(cursor.read_u64()?),
        });
    }
    let human_groups = HumanGroupRegistry::from_parts(human_group_next_id, human_groups)
        .map_err(|_| SaveError::InvalidBinary("invalid HumanGroup registry"))?;

    let settlement_next_id = cursor.read_u64()?;
    let settlement_count = cursor.read_count("settlements")?;
    let mut settlements = Vec::with_capacity(settlement_count);
    for _ in 0..settlement_count {
        settlements.push(SettlementEntity {
            id: SettlementId(cursor.read_u64()?),
        });
    }
    let settlements = SettlementRegistry::from_parts(settlement_next_id, settlements)
        .map_err(|_| SaveError::InvalidBinary("invalid Settlement registry"))?;

    let community_next_id = cursor.read_u64()?;
    let community_count = cursor.read_count("communities")?;
    let mut communities = Vec::with_capacity(community_count);
    for _ in 0..community_count {
        communities.push(CommunityEntity {
            id: CommunityId(cursor.read_u64()?),
        });
    }
    let communities = CommunityRegistry::from_parts(community_next_id, communities)
        .map_err(|_| SaveError::InvalidBinary("invalid Community registry"))?;

    let political_entity_next_id = cursor.read_u64()?;
    let political_entity_count = cursor.read_count("political_entities")?;
    let mut political_entities = Vec::with_capacity(political_entity_count);
    for _ in 0..political_entity_count {
        political_entities.push(PoliticalEntity {
            id: PoliticalEntityId(cursor.read_u64()?),
        });
    }
    let political_entities =
        PoliticalEntityRegistry::from_parts(political_entity_next_id, political_entities)
            .map_err(|_| SaveError::InvalidBinary("invalid PoliticalEntity registry"))?;

    let country_next_id = cursor.read_u64()?;
    let country_count = cursor.read_count("countries")?;
    let mut countries = Vec::with_capacity(country_count);
    for _ in 0..country_count {
        countries.push(CountryEntity {
            id: CountryId(cursor.read_u64()?),
        });
    }
    let countries = CountryRegistry::from_parts(country_next_id, countries)
        .map_err(|_| SaveError::InvalidBinary("invalid Country registry"))?;

    let city_next_id = cursor.read_u64()?;
    let city_count = cursor.read_count("cities")?;
    let mut cities = Vec::with_capacity(city_count);
    for _ in 0..city_count {
        cities.push(CityEntity {
            id: CityId(cursor.read_u64()?),
            country: read_optional_country(cursor)?,
            region: read_optional_region(cursor)?,
        });
    }
    let cities = CityRegistry::from_parts(city_next_id, cities)
        .map_err(|_| SaveError::InvalidBinary("invalid City registry"))?;

    Ok(WorldState {
        date,
        elapsed_days,
        seed,
        terrain,
        resources,
        spatial,
        entities: EntityWorldState {
            human_groups,
            settlements,
            communities,
            political_entities,
            countries,
            cities,
        },
    })
}
'''

new = '''    let entities = read_entity_world(cursor)?;

    Ok(WorldState {
        date,
        elapsed_days,
        seed,
        terrain,
        resources,
        spatial,
        entities,
    })
}

fn read_entity_world(cursor: &mut Cursor<'_>) -> Result<EntityWorldState, SaveError> {
    let human_group_next_id = cursor.read_u64()?;
    let human_group_count = cursor.read_count("human_groups")?;
    let mut human_groups = Vec::with_capacity(human_group_count);
    for _ in 0..human_group_count {
        human_groups.push(HumanGroupEntity {
            id: HumanGroupId(cursor.read_u64()?),
        });
    }
    let human_groups = HumanGroupRegistry::from_parts(human_group_next_id, human_groups)
        .map_err(|_| SaveError::InvalidBinary("invalid HumanGroup registry"))?;

    let settlement_next_id = cursor.read_u64()?;
    let settlement_count = cursor.read_count("settlements")?;
    let mut settlements = Vec::with_capacity(settlement_count);
    for _ in 0..settlement_count {
        settlements.push(SettlementEntity {
            id: SettlementId(cursor.read_u64()?),
        });
    }
    let settlements = SettlementRegistry::from_parts(settlement_next_id, settlements)
        .map_err(|_| SaveError::InvalidBinary("invalid Settlement registry"))?;

    let community_next_id = cursor.read_u64()?;
    let community_count = cursor.read_count("communities")?;
    let mut communities = Vec::with_capacity(community_count);
    for _ in 0..community_count {
        communities.push(CommunityEntity {
            id: CommunityId(cursor.read_u64()?),
        });
    }
    let communities = CommunityRegistry::from_parts(community_next_id, communities)
        .map_err(|_| SaveError::InvalidBinary("invalid Community registry"))?;

    let political_entity_next_id = cursor.read_u64()?;
    let political_entity_count = cursor.read_count("political_entities")?;
    let mut political_entities = Vec::with_capacity(political_entity_count);
    for _ in 0..political_entity_count {
        political_entities.push(PoliticalEntity {
            id: PoliticalEntityId(cursor.read_u64()?),
        });
    }
    let political_entities =
        PoliticalEntityRegistry::from_parts(political_entity_next_id, political_entities)
            .map_err(|_| SaveError::InvalidBinary("invalid PoliticalEntity registry"))?;

    let country_next_id = cursor.read_u64()?;
    let country_count = cursor.read_count("countries")?;
    let mut countries = Vec::with_capacity(country_count);
    for _ in 0..country_count {
        countries.push(CountryEntity {
            id: CountryId(cursor.read_u64()?),
        });
    }
    let countries = CountryRegistry::from_parts(country_next_id, countries)
        .map_err(|_| SaveError::InvalidBinary("invalid Country registry"))?;

    let city_next_id = cursor.read_u64()?;
    let city_count = cursor.read_count("cities")?;
    let mut cities = Vec::with_capacity(city_count);
    for _ in 0..city_count {
        cities.push(CityEntity {
            id: CityId(cursor.read_u64()?),
            country: read_optional_country(cursor)?,
            region: read_optional_region(cursor)?,
        });
    }
    let cities = CityRegistry::from_parts(city_next_id, cities)
        .map_err(|_| SaveError::InvalidBinary("invalid City registry"))?;

    Ok(EntityWorldState {
        human_groups,
        settlements,
        communities,
        political_entities,
        countries,
        cities,
    })
}
'''

if old not in text:
    raise SystemExit("read_world entity block anchor missing")

path.write_text(text.replace(old, new, 1))
