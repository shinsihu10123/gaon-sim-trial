use simulation_model::{
    CommandPayload, EntityRef, EntityRegistry, EntityRegistryError, EventPayload,
    RegionPoliticalState,
};

use crate::SimulationEngine;

impl SimulationEngine {
    /// Applies one entity-lifecycle command at the deterministic daily command
    /// phase. `Ok(None)` means the payload is not an entity mutation.
    pub(crate) fn apply_entity_payload(
        &mut self,
        payload: &CommandPayload,
    ) -> Result<Option<EventPayload>, EntityRegistryError> {
        match payload {
            CommandPayload::NoOp { .. } => Ok(None),
            CommandPayload::CreateHumanGroupEntity
            | CommandPayload::RemoveHumanGroupEntity { .. }
            | CommandPayload::CreateSettlementEntity
            | CommandPayload::RemoveSettlementEntity { .. }
            | CommandPayload::CreateCommunityEntity
            | CommandPayload::RemoveCommunityEntity { .. }
            | CommandPayload::CreatePoliticalEntity
            | CommandPayload::RemovePoliticalEntity { .. }
            | CommandPayload::PromotePoliticalEntityToCountry { .. } => {
                self.apply_pre_state_entity_payload(payload).map(Some)
            }
            CommandPayload::CreateCountryEntity => {
                let id = self.state.entities.countries.create()?;
                Ok(Some(EventPayload::EntityCreated {
                    entity: EntityRef::country(id),
                }))
            }
            CommandPayload::RemoveCountryEntity { country_id } => {
                if self.country_is_referenced(*country_id) {
                    return Err(EntityRegistryError::ReferenceInUse);
                }
                self.state.entities.countries.remove(*country_id)?;
                Ok(Some(EventPayload::EntityRemoved {
                    entity: EntityRef::country(*country_id),
                }))
            }
            CommandPayload::CreateCityEntity {
                country_id,
                region_id,
            } => {
                if country_id.is_some_and(|id| !self.state.entities.countries.contains(id))
                    || region_id.is_some_and(|id| self.state.spatial.region(id).is_none())
                {
                    return Err(EntityRegistryError::InvalidReference);
                }
                let id = self.state.entities.cities.create(*country_id, *region_id)?;
                Ok(Some(EventPayload::EntityCreated {
                    entity: EntityRef::city(id),
                }))
            }
            CommandPayload::RemoveCityEntity { city_id } => {
                self.state.entities.cities.remove(*city_id)?;
                Ok(Some(EventPayload::EntityRemoved {
                    entity: EntityRef::city(*city_id),
                }))
            }
            CommandPayload::CreateRegionEntity { draft } => {
                if political_references_unknown_country(
                    draft.political,
                    &self.state.entities.countries,
                ) {
                    return Err(EntityRegistryError::InvalidReference);
                }
                let id = self
                    .state
                    .spatial
                    .create_region(draft.clone())
                    .map_err(|_| EntityRegistryError::InvalidReference)?;
                Ok(Some(EventPayload::EntityCreated {
                    entity: EntityRef::region(id),
                }))
            }
            CommandPayload::RemoveRegionEntity { region_id } => {
                if self.region_is_referenced(*region_id) {
                    return Err(EntityRegistryError::ReferenceInUse);
                }
                self.state
                    .spatial
                    .remove_region(*region_id)
                    .map_err(|_| EntityRegistryError::InvalidReference)?;
                Ok(Some(EventPayload::EntityRemoved {
                    entity: EntityRef::region(*region_id),
                }))
            }
        }
    }

    fn apply_pre_state_entity_payload(
        &mut self,
        payload: &CommandPayload,
    ) -> Result<EventPayload, EntityRegistryError> {
        match payload {
            CommandPayload::CreateHumanGroupEntity => {
                let id = self.state.entities.human_groups.create()?;
                Ok(EventPayload::EntityCreated {
                    entity: EntityRef::human_group(id),
                })
            }
            CommandPayload::RemoveHumanGroupEntity { human_group_id } => {
                self.state.entities.human_groups.remove(*human_group_id)?;
                Ok(EventPayload::EntityRemoved {
                    entity: EntityRef::human_group(*human_group_id),
                })
            }
            CommandPayload::CreateSettlementEntity => {
                let id = self.state.entities.settlements.create()?;
                Ok(EventPayload::EntityCreated {
                    entity: EntityRef::settlement(id),
                })
            }
            CommandPayload::RemoveSettlementEntity { settlement_id } => {
                self.state.entities.settlements.remove(*settlement_id)?;
                Ok(EventPayload::EntityRemoved {
                    entity: EntityRef::settlement(*settlement_id),
                })
            }
            CommandPayload::CreateCommunityEntity => {
                let id = self.state.entities.communities.create()?;
                Ok(EventPayload::EntityCreated {
                    entity: EntityRef::community(id),
                })
            }
            CommandPayload::RemoveCommunityEntity { community_id } => {
                self.state.entities.communities.remove(*community_id)?;
                Ok(EventPayload::EntityRemoved {
                    entity: EntityRef::community(*community_id),
                })
            }
            CommandPayload::CreatePoliticalEntity => {
                let id = self.state.entities.political_entities.create()?;
                Ok(EventPayload::EntityCreated {
                    entity: EntityRef::political_entity(id),
                })
            }
            CommandPayload::RemovePoliticalEntity {
                political_entity_id,
            } => {
                self.state
                    .entities
                    .political_entities
                    .remove(*political_entity_id)?;
                Ok(EventPayload::EntityRemoved {
                    entity: EntityRef::political_entity(*political_entity_id),
                })
            }
            CommandPayload::PromotePoliticalEntityToCountry {
                political_entity_id,
            } => {
                if !self
                    .state
                    .entities
                    .political_entities
                    .contains(*political_entity_id)
                {
                    return Err(EntityRegistryError::UnknownEntity);
                }
                let country_id = self.state.entities.countries.create()?;
                self.state
                    .entities
                    .political_entities
                    .remove(*political_entity_id)?;
                Ok(EventPayload::EntityTransitioned {
                    from: EntityRef::political_entity(*political_entity_id),
                    to: EntityRef::country(country_id),
                })
            }
            _ => unreachable!("pre-state lifecycle dispatcher received non-pre-state payload"),
        }
    }

    fn country_is_referenced(&self, country_id: simulation_model::CountryId) -> bool {
        self.state.spatial.regions.iter().any(|region| {
            region.political.legal_owner == Some(country_id)
                || region.political.controller == Some(country_id)
        }) || self
            .state
            .entities
            .cities
            .iter()
            .any(|city| city.country == Some(country_id))
            || self.pending_commands.iter().any(|command| {
                matches!(
                    command.source,
                    simulation_model::CommandSource::CountryAi(id) if id == country_id
                ) || command.payload.references_country(country_id)
            })
    }

    fn region_is_referenced(&self, region_id: simulation_model::RegionId) -> bool {
        self.state
            .entities
            .human_groups
            .iter()
            .any(|group| {
                group
                    .initial
                    .as_ref()
                    .is_some_and(|initial| initial.region_id == region_id)
            })
            || self
                .state
                .entities
                .cities
                .iter()
                .any(|city| city.region == Some(region_id))
            || self
                .pending_commands
                .iter()
                .any(|command| command.payload.references_region(region_id))
    }
}

fn political_references_unknown_country(
    political: RegionPoliticalState,
    countries: &simulation_model::CountryRegistry,
) -> bool {
    political
        .legal_owner
        .is_some_and(|country| !countries.contains(country))
        || political
            .controller
            .is_some_and(|country| !countries.contains(country))
}
