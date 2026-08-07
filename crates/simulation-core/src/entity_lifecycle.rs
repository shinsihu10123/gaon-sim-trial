use simulation_model::{
    CommandPayload, EntityRef, EntityRegistry, EntityRegistryError, EventPayload, RegionPoliticalState,
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
