#![deny(warnings)]
use warp::Filter;
use mobc::Pool;
use mobc_postgres::{tokio_postgres, PgConnectionManager};
use tokio_postgres::NoTls;
use std::convert::Infallible;
use mobc_postgres::mobc::Connection;
use std::env;

type DBCon = Connection<PgConnectionManager<NoTls>>;
type DBPool = Pool<PgConnectionManager<NoTls>>;

mod controllers;
pub mod models;
mod router;
mod db;

fn json_body() -> impl Filter<Extract = (models::team::Team,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

fn with_db(db_pool: DBPool) -> impl Filter<Extract = (DBPool,), Error = Infallible> + Clone {
    warp::any().map(move || db_pool.clone())
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let logger = warp::log("user-service");

    let pg = db::pool().await.unwrap();

    let ok = warp::get().and(warp::path::end()).map(|| "ok");

    let list_users = warp::path!("users")
        .and(warp::get())
        .and(warp::path::end())
        .and(with_db(pg.clone()))
        .and_then(controllers::team_controller::list_teams)
        .boxed();

    let get_user = warp::path!("users" / String)
        .and(warp::get())
        .and(warp::path::end())
        .and(with_db(pg.clone()))
        .and_then(|_uuid: String, _pg: DBPool| controllers::team_controller::get_team(_uuid, _pg))
        .boxed();

    let create_user = warp::path!("users")
        .and(warp::post())
        .and(warp::path::end())
        .and(json_body())
        .and(with_db(pg.clone()))
        .and_then(|_team: models::team::Team, _pg: DBPool| controllers::team_controller::create_team(_team, _pg))
        .boxed();

    let update_user = warp::path!("users" / String)
        .and(warp::patch())
        .and(warp::path::end())
        .and(json_body())
        .and(with_db(pg.clone()))
        .and_then(|_uuid: String, _team: models::team::Team, _pg: DBPool| controllers::team_controller::update_team(_uuid, _team, _pg))
        .boxed();

    let delete_user = warp::path!("users" / String)
        .and(warp::delete())
        .and(warp::path::end())
        .and(with_db(pg.clone()))
        .and_then(|_uuid: String, _pg: DBPool| controllers::team_controller::delete_team(_uuid, _pg))
        .boxed();

    let router = ok.or(list_users)
                                            .or(get_user)
                                            .or(create_user)
                                            .or(update_user)
                                            .or(delete_user)
                                            .with(logger);


    let _port: i32 = match env::var("SERVICE_PORT") {
        Some(var) => var,
        Err(err) => {
            panic!("environment variable SERVICE_PORT is not defined")
        }
    };
    println!("{}",format!("running server at 127.0.0.1:{} ...",_port));
    warp::serve(router).run(([127, 0, 0, 1], _port)).await;




}