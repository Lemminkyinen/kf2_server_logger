// #[cfg(test)]
// mod database_operations_tests {
//     use crate::kf2_database::models::IpAddressDbQ;
//     use crate::schema::ip_addresses::dsl::*;
//     use crate::{
//         args::Kf2DbArgs, kf2_database::management::KfDbManager, kf2_scrape::models::PlayerInfo,
//     };
//     use diesel::RunQueryDsl;

//     fn get_connection() -> KfDbManager {
//         let args = Kf2DbArgs {
//             server_address: String::from(""),
//             database: String::from("server_test"),
//             username: String::from(""),
//             password: String::from(""),
//         };
//         KfDbManager::new_session(args).unwrap()
//     }

//     #[test]
//     fn add_ip_addresses() {
//         let kf2db = get_connection();
//         let mut conn = kf2db.get_connection().unwrap();
//         let players = vec![
//             PlayerInfo {
//                 steam_id: 1,
//                 ip: std::net::IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 52)),
//                 name: "kissa".to_string(),
//                 ping: 120,
//                 unique_net_id: "kissa123".to_string(),
//                 admin: false,
//             },
//             PlayerInfo {
//                 steam_id: 2,
//                 ip: std::net::IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 51)),
//                 name: "koira".to_string(),
//                 ping: 130,
//                 unique_net_id: "koira123".to_string(),
//                 admin: false,
//             },
//         ];
//         KfDbManager::insert_unique_players(&mut conn, players.clone()).unwrap();
//         KfDbManager::insert_ip_addresses(&mut conn, players).unwrap();
//         let result = ip_addresses.load::<IpAddressDbQ>(&mut conn).unwrap();
//         assert_eq!(result.len(), 2);
//     }
// }
