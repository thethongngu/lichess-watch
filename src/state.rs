use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Color {
    BLACK,
    WHITE,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct UserInfo {
    pub name: String,
    pub title: Option<String>,
    pub id: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Player {
    pub color: Color,
    pub rating: i32,
    pub seconds: i32,

    #[serde(alias = "user")]
    pub user_info: UserInfo,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Participant {
    pub white_player: Player,
    pub black_player: Player,
}

fn choose_color<'de, D>(deserializer: D) -> Result<Participant, D::Error>
where
    D: Deserializer<'de>,
{
    let mut players: Vec<Player> = Deserialize::deserialize(deserializer)?;
    if players[0].color == Color::BLACK {
        Ok(Participant { white_player: players.swap_remove(1), black_player: players.swap_remove(0) })
    } else {
        Ok(Participant { black_player: players.swap_remove(1), white_player: players.swap_remove(0) })
    }
}

fn split_fen_only<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let mut res: String = Deserialize::deserialize(deserializer)?;
    res.pop();
    res.pop();
    Ok(res)
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(tag = "t", content = "d")]
pub enum FeedResponse {
    #[serde(alias = "featured")]
    Feature {
        id: String,
        orientation: Color,

        #[serde(deserialize_with = "choose_color")]
        #[serde(alias = "players")]
        players: Participant,

        fen: String,
    },

    #[serde(alias = "fen")]
    Fen {
        #[serde(deserialize_with = "split_fen_only")]
        fen: String,

        lm: String,
        wc: i32,
        bc: i32,
    },
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::state::Participant;
    use crate::state::Player;
    use crate::state::UserInfo;

    use super::Color;
    use super::FeedResponse;

    #[test]
    fn parse_fen_response() {
        let json_str: &'static str = "{\"t\":\"fen\",\"d\":{\"fen\":\"4rbk1/1b1q1pp1/1n2p3/2ppP1BP/\
                                1nP3N1/1P3NP1/5PB1/rQ2R1K1 w\",\"lm\":\"a8a1\",\"wc\":37,\"bc\":30}}";
        let raw: Bytes = Bytes::from(json_str);
        let json: Result<FeedResponse, serde_json::Error> = serde_json::from_slice(&raw);

        assert_eq!(
            json.unwrap(),
            FeedResponse::Fen {
                fen: String::from("4rbk1/1b1q1pp1/1n2p3/2ppP1BP/1nP3N1/1P3NP1/5PB1/rQ2R1K1 w"),
                lm: String::from("a8a1"),
                wc: 37,
                bc: 30
            }
        );
    }

    #[test]
    fn parse_feature_response() {
        let json_str: &'static str = "{\"t\":\"featured\",\"d\":{\"id\":\"1n8qK1ar\",\"orientation\":\"white\",\"players\":[{\"color\":\"white\",\"user\":{\"name\":\"Aqua_Blazing\",\"id\":\"aqua_blazing\"},\"rating\":2965,\"seconds\":60},{\"color\":\"black\",\"user\":{\"name\":\"Player_06\",\"title\":\"FM\",\"id\":\"player_06\"},\"rating\":2945,\"seconds\":60}],\"fen\":\"r3rbk1/1b1q1pp1/1n2p3/2ppP1BP/1nP3N1/1P3NP1/5PB1/RQ2R1K1\"}}";
        let raw: Bytes = Bytes::from(json_str);
        let json: Result<FeedResponse, serde_json::Error> = serde_json::from_slice(&raw);

        assert_eq!(
            json.unwrap(),
            FeedResponse::Feature {
                id: String::from("1n8qK1ar"),
                orientation: Color::WHITE,
                players: Participant {
                    white_player: Player {
                        color: Color::WHITE,
                        rating: 2965,
                        seconds: 60,
                        user_info: UserInfo {
                            id: String::from("aqua_blazing"),
                            name: String::from("Aqua_Blazing"),
                            title: None,
                        }
                    },
                    black_player: Player {
                        color: Color::BLACK,
                        rating: 2945,
                        seconds: 60,
                        user_info: UserInfo {
                            id: String::from("player_06"),
                            name: String::from("Player_06"),
                            title: Some(String::from("FM")),
                        }
                    },
                },
                fen: String::from("r3rbk1/1b1q1pp1/1n2p3/2ppP1BP/1nP3N1/1P3NP1/5PB1/RQ2R1K1")
            }
        );
    }

    #[test]
    fn parse_wrong_format_response() {
        let json_str: &'static str = "{\"t\":orientation\":\"white\",\"players\":[{\"color\":\"white\",\"user\":{\"name\":\"Aqua_Blazing\",\"id\":\"aqua_blazing\"},\"rating\":2965,\"seconds\":60},{\"color\":\"black\",\"user\":{\"name\":\"Player_06\",\"title\":\"FM\",\"id\":\"player_06\"},\"rating\":2945,\"seconds\":60}],\"fen\":\"r3rbk1/1b1q1pp1/1n2p3/2ppP1BP/1nP3N1/1P3NP1/5PB1/RQ2R1K1\"}}";
        let raw: Bytes = Bytes::from(json_str);
        let json: Result<FeedResponse, serde_json::Error> = serde_json::from_slice(&raw);

        assert_eq!(json.is_err(), true);
    }

    #[test]
    fn parse_feature_response_with_missing_field() {
        let json_str: &'static str = "{\"t\":\"featured\",\"d\":{\"id\":\"1n8qK1ar\",\"orientation\":\"white\",\"players\":[{\"color\":\"white\",\"rating\":2965,\"seconds\":60},{\"color\":\"black\",\"user\":{\"name\":\"Player_06\",\"title\":\"FM\",\"id\":\"player_06\"},\"rating\":2945,\"seconds\":60}],\"fen\":\"r3rbk1/1b1q1pp1/1n2p3/2ppP1BP/1nP3N1/1P3NP1/5PB1/RQ2R1K1\"}}";
        let raw: Bytes = Bytes::from(json_str);
        let json: Result<FeedResponse, serde_json::Error> = serde_json::from_slice(&raw);

        assert_eq!(json.is_err(), true);
    }
}
