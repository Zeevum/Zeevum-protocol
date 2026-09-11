# zeevum-protocol

Wire protocol of the *Zeevum* messenger: message types, encoding, proof of work.

A single contract shared by the server and its clients, so a mismatch between
the two is caught by the compiler rather than by a user with a broken app.

## Transport

- Line-delimited JSON (UTF-8): one message per frame, terminated by `\n`.
- Serialization escapes newlines, so message content may contain any character,
  including `\n`.
- Hard frame limit: *64 KiB* (`MAX_LINE_BYTES`). Exceeding it closes the
  connection.
- `\r` and `\n` are tolerated (Windows compatibility).
- An unknown message type is a deserialization error: incompatible versions
  cannot silently misread each other.
- The first frame of every connection is `ClientMsg::Auth`, which carries
  `protocol_version`. The server rejects incompatible versions explicitly.

### Frame example

```
{"type":"send_msg","message_id":"0194...","conv_id":"6f1c...","content":"Hi"}
```

## Identity model

Two identifiers, and keeping them apart is the point of this crate:

| Type | Meaning | Properties |
|---|---|---|
| `UserId` (i64) | *who* a user is | Public, stable, searchable by login |
| `ConvId` (UUID) | *where* a message goes | Opaque to clients, issued by the server |

A conversation is a first-class object: a 1:1 chat today, a group in the future.
Clients never derive or guess a `ConvId`; they obtain one from the server
(`DmResolved`, `FriendList`, group creation) and store it.

Messages are addressed by `conv_id`, never by peer. That is what keeps group
chats and end-to-end encryption additive instead of breaking changes: both need
"the conversation" to be distinct from "the other participant".

## Compatibility

| Protocol version | Status | Used by |
|---|---|---|
| v1 | Deprecated | `Zeevum-server` up to v0.3.x, `Zeevum-android` up to v0.1.x |
| v2 | Current | Under development, see the `dev` branch |

### Versioning rules

`PROTOCOL_VERSION` is incremented on any breaking change to the message types:
renaming a field, changing a type, removing a variant. Adding a variant breaks
older receivers too, so it also counts as breaking.

Server and clients ship in the same release. Between releases the two may drift;
an incompatible pair gets an explicit handshake error instead of silent
misbehaviour.

## Client to server

| Type | Fields | Purpose |
|---|---|---|
| `auth` | `protocol_version`, `method` | First frame after TLS. `method` is `token` / `login` / `register` |
| `pow_solution` | `nonce` | Answer to `pow_challenge` during registration |
| `search_user` | `login` | Find a user by login |
| `friend_req` | `target_user_id` | Send a friend request |
| `accept_friend` | `target_user_id` | Accept a friend request |
| `resolve_dm` | `peer_user_id` | Get (or lazily create) the 1:1 conversation |
| `history_req` | `conv_id` | Up to `HISTORY_LIMIT` recent messages |
| `send_msg` | `message_id`, `conv_id`, `content` | Send a message. The client generates `message_id` |
| `mark_read` | `message_id` | Mark a message as read |

## Server to client

| Type | Fields | Purpose |
|---|---|---|
| `pow_challenge` | `challenge`, `difficulty_bits` | Proof of work requested before registration |
| `auth_ok` | `user_id`, `token`, `expires_at` | Successful authentication; token for auto-login |
| `error` | `code`, `detail` | Rejection, including incompatible protocol versions |
| `friend_list` | `entries: UserBrief[]` | Starting state: friends |
| `pending_reqs` | `entries: UserBrief[]` | Starting state: incoming requests |
| `incoming_req` | `from: UserBrief` | Online notification of a new request |
| `friend_added` | `user: UserBrief` | A request was accepted (both sides) |
| `friend_req_sent` | `user: UserBrief` | Your request was stored |
| `user_found` / `user_not_found` | `user` | Result of `search_user` |
| `dm_resolved` | `conv_id`, `peer: UserBrief` | Result of `resolve_dm` |
| `history_msg` | `message_id`, `conv_id`, `sender_user_id`, `timestamp`, `content`, `is_read` | One history message; batch ends with `history_end` |
| `history_end` | `conv_id` | End of a history batch |
| `msg_ack` | `message_id`, `conv_id` | The server stored the message |
| `recv_msg` | `message_id`, `conv_id`, `sender_user_id`, `timestamp`, `content` | Incoming message |
| `msg_read` | `message_id`, `conv_id` | The recipient read your message |

> `UserBrief = { user_id: i64, login: string }`.

## Errors

Errors carry a machine-readable `code`, never a message the client has to parse:

```json
{"type":"error","code":"not_a_member","detail":"conv 6f1c... user 42"}
```

`code` values: `unsupported_protocol_version`, `invalid_credentials`,
`registration_failed`, `malformed_frame`, `not_a_member`, `no_pending_request`,
`already_friends`, `cannot_target_yourself`, `conversation_not_found`,
`user_not_found`, `message_too_long`, `internal`.

`detail` is for logs and debugging. Clients localize on `code` and never show
`detail` to the user as-is.

## Session rules

1. A token is issued on every successful authentication.
2. It is extended on every use (sliding expiry).
3. Lifetime is configured by the server.
4. The client stores the token and passes it back in `auth.method.token`.

## Proof of work

Applied to registration only (anti-spam, not a targeted defense):

The server sends `pow_challenge`; `challenge` is 16 hex characters plus
`difficulty_bits`. The client searches for a `nonce` such that
`sha256(challenge, decimal_nonce)` has at least `difficulty_bits` leading zero
bits, then answers with `pow_solution`.

Server difficulties: `weak` = 16 bits (~65k attempts), `medium` = 20 bits
(~1M), `strong` = 24 bits (~16M).

> Solving is CPU-bound: run it off the async runtime.

## Usage

```toml
[dependencies]
zeevum-protocol = { git = "https://github.com/Zeevum/zeevum-protocol", tag = "v0.2.0" }
```

```rust
use zeevum_protocol::{ClientMsg, encode, decode};

let frame = encode(&ClientMsg::SendMsg {
    message_id: uuid::Uuid::new_v4(),
    conv_id,
    content: "Hi".into(),
})?; // ready to write, ends with '\n'

let msg: ClientMsg = decode(&line)?;
```

## License

[MIT](LICENSE)
