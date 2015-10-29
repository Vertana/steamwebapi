extern crate hyper;
extern crate serde;
extern crate serde_json;

use std::io::prelude::*;

#[derive(Debug)]
pub enum Error {
    HttpError(hyper::error::Error),
    Io(std::io::Error),
    BadBody,
    SteamError(u32) // EResult
}
impl std::convert::From<hyper::error::Error> for Error {
    fn from(e: hyper::error::Error) -> Error {
        Error::HttpError(e)
    }
}
impl std::convert::From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Error {
        Error::BadBody
    }
}
impl std::convert::From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::Io(e)
    }
}

pub struct ApiClient {
    http: hyper::client::Client,
    apikey: String
}
impl ApiClient {
    pub fn new(apikey: String) -> ApiClient {
        ApiClient {
            http: hyper::client::Client::new(),
            apikey: apikey 
        }
    }
    pub fn get_player_summary(&self, steamid: u64) -> Result<serde_json::Value, Error> { 
        let steamids_str = steamid.to_string(); 
        let endpoint = {
            let mut endpoint = hyper::Url::parse("http://api.steampowered.com/ISteamUser/GetPlayerSummaries/v0002/").unwrap();
            endpoint.set_query_from_pairs(vec![
                                          ("key", &self.apikey as &str),
                                          ("steamids", &steamids_str)
                                          ].into_iter());
            endpoint
        };

        let body = {
            let mut response = try!(self.http.get(endpoint).send());
            let mut body = String::new();
            try!(response.read_to_string(&mut body));
            body
        };

        let json = try!(serde_json::from_str::<serde_json::Value>(&body));
        let player = json.as_object().and_then(|o| o.get("response"))
            .and_then(|response| response.as_object())
            .and_then(|o| o.get("players"))
            .and_then(|players| players.as_array())
            .and_then(|a| a.get(0));
        if let Some(player) = player {
            Ok(player.clone())
        } else {
            Err(Error::BadBody)
        }
    }
    pub fn get_player_server(&self, steamid: u64) -> Result<Option<String>, Error> {
        let player = try!(self.get_player_summary(steamid));

        if let Some(player) = player.as_object() {
            if player.get("gameid").and_then(|gameid| gameid.as_string()) == Some("440") {
                return Ok(player.get("gameserverip").and_then(|ip| ip.as_string()).map(|x| x.to_owned()))
            }
        }

        return Ok(None)
    }
}
