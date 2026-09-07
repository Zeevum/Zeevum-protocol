use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod pow;

/// Текущая версия протокола
pub const PROTOCOL_VERSION: u32 = 1;
pub const MAX_LOGIN_LEN: usize = 32;
pub const MAX_MESSAGE_LEN: usize = 4096;
/// Защита от DoS
pub const MAX_LINE_BYTES: usize = 64 * 1024;
/// Сколько сообщений истории отдаётся за один HistoryReq
pub const HISTORY_LIMIT: i64 = 50;
/// Идентификатор пользователя
pub type ChatId = i64;

/// Краткое представление пользователя
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserBrief {
    pub chat_id: ChatId,
    pub login: String,
}

/// Сообщение от клиента к серверу
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMsg {
    Auth {
        protocol_version: u32,
        method: AuthMethod,
    },
    PowSolution {
        nonce: u64,
    },
    SearchUser {
        login: String,
    },
    FriendReq {
        target_chat_id: ChatId,
    },
    AcceptFriend {
        target_chat_id: ChatId,
    },
    HistoryReq {
        peer_chat_id: ChatId,
    },
    SendMsg {
        message_id: Uuid,
        peer_chat_id: ChatId,
        content: String,
    },
    MarkRead {
        message_id: Uuid,
    },
}

/// Способ авторизации. Для Register требуется прохождение PoW
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AuthMethod {
    Token {
        token: String,
    },
    Login {
        login: String,
        password: String,
    },
    Register {
        login: String,
        password: String,
    },
}

/// Сообщение от сервера к клиенту
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMsg {
    PowChallenge {
        challenge: String,
        difficulty_bits: u32,
    },
    AuthOk {
        chat_id: ChatId,
        token: String,
        expires_at: i64,
    },
    AuthFailed {
        reason: String,
    },
    FriendList {
        entries: Vec<UserBrief>,
    },
    PendingReqs {
        entries: Vec<UserBrief>,
    },
    IncomingReq {
        from: UserBrief,
    },
    FriendAdded {
        user: UserBrief,
    },
    UserFound {
        user: UserBrief,
    },
    UserNotFound,
    Info {
        text: String,
    },
    HistoryMsg {
        message_id: Uuid,
        sender_chat_id: ChatId,
        timestamp: i64,
        content: String,
        is_read: bool,
    },
    HistoryEnd,
    MsgAck {
        message_id: Uuid,
    },
    RecvMsg {
        message_id: Uuid,
        chat_id: Uuid,
        sender_chat_id: ChatId,
        timestamp: i64,
        content: String,
    },
    MsgRead {
        message_id: Uuid,
    },
}

pub fn encode<T: Serialize>(msg: &T) -> serde_json::Result<String> {
    let mut line = serde_json::to_string(msg)?;
    line.push('\n');
    Ok(line)
}

/// Разбор строки фрейма
pub fn decode<T: serde::de::DeserializeOwned>(line: &str) -> serde_json::Result<T> {
    serde_json::from_str(line.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_auth_login() {
        let msg = ClientMsg::Auth {
            protocol_version: PROTOCOL_VERSION,
            method: AuthMethod::Login {
                login: "alice".into(),
                password: "p@ss w0rd!".into(),
            },
        };
        let line = encode(&msg).unwrap();
        assert!(line.ends_with('\n'));
        assert_eq!(line.matches('\n').count(), 1);
        let back: ClientMsg = decode(&line).unwrap();
        assert_eq!(back, msg);
    }

    #[test]
    fn newline_and_unicode_survive() {
        let msg = ServerMsg::RecvMsg {
            message_id: Uuid::new_v4(),
            chat_id: Uuid::new_v4(),
            sender_chat_id: 1234567,
            timestamp: 1730000000,
            content: "строка1\nстрока2\t😀 \"кавычки\"".into(),
        };
        let line = encode(&msg).unwrap();
        assert_eq!(line.matches('\n').count(), 1);
        let back: ServerMsg = decode(&line).unwrap();
        assert_eq!(back, msg);
    }

    #[test]
    fn unknown_type_is_rejected() {
        let result: Result<ClientMsg, _> = decode(r#"{"type":"mind_control","x":1}"#);
        assert!(result.is_err());
    }

    #[test]
    fn crlf_tolerated() {
        let msg = ServerMsg::UserNotFound;
        let line = encode(&msg).unwrap();
        let with_crlf = format!("{}\r", line);
        let back: ServerMsg = decode(&with_crlf).unwrap();
        assert_eq!(back, msg);
    }

    #[test]
    fn history_msg_carries_read_state() {
        let msg = ServerMsg::HistoryMsg {
            message_id: Uuid::new_v4(),
            sender_chat_id: 7654321,
            timestamp: 1730000001,
            content: "прочитано".into(),
            is_read: true,
        };
        let back: ServerMsg = decode(&encode(&msg).unwrap()).unwrap();
        assert_eq!(back, msg);
    }

    #[test]
    fn roundtrip_pow_solution() {
        let msg = ClientMsg::PowSolution { nonce: 987_654_321 };
        let back: ClientMsg = decode(&encode(&msg).unwrap()).unwrap();
        assert_eq!(back, msg);
    }
}