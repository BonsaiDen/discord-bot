// STD Dependencies -----------------------------------------------------------
use std::collections::HashMap;


// External Dependencies ------------------------------------------------------
use diesel;
use diesel::prelude::*;


// Internal Dependencies ------------------------------------------------------
use super::super::Server;
use ::db::models::{Alias, NewAlias};
use ::db::schema::aliases::dsl::{server_id, name as alias_name};
use ::db::schema::aliases::table as aliasTable;


// Server Aliases Interface ---------------------------------------------------
impl Server {

    pub fn has_alias(&self, name: &str) -> bool {
        aliasTable.filter(
            server_id.eq(&self.config.table_id)

        ).filter(
            alias_name.eq(name)

        ).count().get_result(&self.config.connection).unwrap_or(0) > 0
    }

    pub fn add_alias(&mut self, name: &str, effect_names: &[String]) {
        diesel::insert(&NewAlias {
            server_id: &self.config.table_id,
            name: name,
            effect_names: &effect_names.join(" ")

        }).into(aliasTable).execute(&self.config.connection).and_then(|_| {
            Ok(self.update_aliases())

        }).ok();
    }

    pub fn remove_alias(&mut self, name: &str) {
        diesel::delete(
            aliasTable.filter(
                server_id.eq(&self.config.table_id)

            ).filter(
                alias_name.eq(name)
            )

        ).execute(&self.config.connection).and_then(|_| {
            Ok(self.update_aliases())

        }).ok();
    }

    pub fn list_aliases(&self) -> Vec<Alias> {
        aliasTable.filter(
            server_id.eq(&self.config.table_id)

        ).order(alias_name).load::<Alias>(
            &self.config.connection

        ).unwrap_or_else(|_| vec![])
    }

    pub fn update_aliases(&mut self) {

        self.aliases.clear();

        for alias in aliasTable.filter(
            server_id.eq(&self.config.table_id)

        ).load::<Alias>(
            &self.config.connection

        ).unwrap_or_else(|_| vec![]) {
            self.aliases.insert(
                alias.name,
                alias.effect_names.split(' ').map(|s| s.to_string()).collect()
            );
        }

    }

    pub fn get_alias_map(&self) -> &HashMap<String, Vec<String>> {
        &self.aliases
    }

}

