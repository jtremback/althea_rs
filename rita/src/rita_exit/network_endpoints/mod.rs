use althea_types::ExitClientIdentity;

use actix::registry::SystemService;
use actix_web::*;

use futures::Future;

use rita_exit::db_client::{DbClient, SetupClient};

use std::boxed::Box;

use serde_json;

use bytes::Bytes;

use settings::{RitaCommonSettings, RitaExitSettings};
use SETTING;

use althea_types::interop::ExitServerIdentity;
use exit_db::models::Client;
use failure::Error;
use rita_exit::db_client::ListClients;

pub fn setup_request(
    req: HttpRequest,
) -> Box<Future<Item = Json<ExitServerIdentity>, Error = Error>> {
    req.body()
        .from_err()
        .and_then(move |bytes: Bytes| {
            trace!("setup request body: {:?}", bytes);
            let their_id: ExitClientIdentity = serde_json::from_slice(&bytes[..]).unwrap();

            trace!("Received requester identity, {:?}", their_id);
            DbClient::from_registry()
                .send(SetupClient(their_id))
                .from_err()
                .and_then(move |reply| {
                    Ok(Json(ExitServerIdentity {
                        own_local_ip: reply.unwrap(),
                        server_local_ip: SETTING.get_exit_network().own_internal_ip,
                        wg_port: SETTING.get_exit_network().wg_tunnel_port,
                        global: SETTING.get_identity(),
                        price: SETTING.get_exit_network().exit_price,
                        netmask: SETTING.get_exit_network().netmask,
                    }))
                })
        })
        .responder()
}

pub fn list_clients(req: HttpRequest) -> Box<Future<Item = Json<Vec<Client>>, Error = Error>> {
    req.body()
        .from_err()
        .and_then(move |_: Bytes| {
            DbClient::from_registry()
                .send(ListClients {})
                .from_err()
                .and_then(move |reply| Ok(Json(reply.unwrap())))
        })
        .responder()
}
