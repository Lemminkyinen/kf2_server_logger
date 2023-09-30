use super::models::KfDbManager;
use super::models_db::PlayerDb;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use log::{debug, error, info, log_enabled, warn, Level};
use std::error::Error;

impl KfDbManager {
    pub async fn log_unique_players(
        &mut self,
        players: Vec<PlayerDb>,
    ) -> Result<(), Box<dyn Error>> {
        use crate::schema::players_::dsl::*;
        let connection = self.get_connection();

        let mut existing_players_db = players_
            .filter(steam_id.eq_any(players.iter().map(|p| &p.steam_id)))
            .load::<PlayerDb>(connection)
            .await?;

        let mut existing_players = players
            .clone()
            .into_iter()
            .filter(|p| {
                existing_players_db
                    .iter()
                    .any(|ep| ep.steam_id == p.steam_id)
            })
            .collect::<Vec<_>>();

        let new_players = players
            .into_iter()
            .filter(|p| {
                !existing_players_db
                    .iter()
                    .any(|ep| ep.steam_id == p.steam_id)
            })
            .collect::<Vec<_>>();

        if !new_players.is_empty() {
            let len = new_players.len();
            diesel::insert_into(players_)
                .values(new_players)
                .execute(connection)
                .await?;
            info!("Added {} new unique players", len);
        }

        if existing_players.len() == existing_players_db.len() {
            existing_players.sort_by(|a, b| a.steam_id.cmp(&b.steam_id));
            existing_players_db.sort_by(|a, b| a.steam_id.cmp(&b.steam_id));

            let zipped = existing_players
                .into_iter()
                .zip(existing_players_db.into_iter());

            for (player, player_db) in zipped {
                if player.steam_id != player_db.steam_id {
                    error!(
                        "Player steam id mismatch: {} != {}",
                        player.steam_id, player_db.steam_id
                    );
                }
                let p_name = player.name;
                let p_count = player_db.count + 1;
                let p_ping = if player_db.ping == 0 {
                    player.ping
                } else {
                    (player_db.ping + player.ping) / 2
                };
                diesel::update(players_.find(player.steam_id))
                    .set((
                        name.eq(p_name),
                        count.eq(p_count),
                        ping.eq(p_ping),
                        ip_address.eq(player.ip_address),
                        unique_net_id.eq(player.unique_net_id),
                        // last_joined.eq(player.last_joined),
                    ))
                    .execute(connection)
                    .await?;
            }
        }

        Ok(())
    }
}
