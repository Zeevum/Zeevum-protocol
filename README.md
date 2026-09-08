# MUC-Protocol
Протокол мессенджера *Zeevum*: типы сообщений, кодирование, Proof-of-Work

Единый контракт между сервером и клиентами — расхождение формата ловится компилятором, а не во время работы клиента и сервера

## Транспорт
- Построчный JSON (UTF-8): каждое сообщение — один фрейм, завершённый переносом строки (`\n`)
- Сериализация экранирует переводы строк, поэтому содержимое сообщений может содержать любые символы, включая `\n`
- Жёсткий лимит фрейма: *64 КиБ* (MAX_LINE_BYTES) — соединение закрывается при превышении
- `\r` и `\n` tolerated (Windows-совместимость)
- Неизвестный тип сообщения это ошибка десериализации: несовместимые версии не могут молча перепутаться
- Первое сообщение любого соединения — ClientMsg::Auth; в нём передаётся `protocol_version`, сервер отклоняет несовместимые версии явной ошибкой

### Пример фрейма
`{"type":"send_msg","message_id":"0194...","peer_chat_id":1234567,"content":"Привет"}`

## Совместимость
| Версия протокола | Статус | Используется в |
|---|---|---|
| v1 | Current | `Zeevum-server v0.2.x` и `Zeevum-android v0.1.x-rc.x` |

### Правила версионирования:
***PROTOCOL_VERSION*** увеличивается при любом ломающем изменении типов (переименование поля, изменение типа, удаление варианта)

Добавление нового варианта ломает старых получателей и потому тоже увеличивает версию

Сервер и клиенты обновляются в одном релизе. Между релизами возможен разнобой — несовместимая пара сторон получает явную ошибку рукопожатия

## Сообщения клиента - серверу
| Тип	| Поля	| Назначение |
|---|---|---|
| auth |	protocol_version, method | Первая строка после TLS. method - token / login / register |
| pow_solution | nonce | Ответ на pow_challenge при регистрации |
| search_user	| login |	Поиск пользователя по логину |
| friend_req |	target_chat_id |	Заявка в друзья |
| accept_friend |	target_chat_id |	Принять заявку |
| history_req |	peer_chat_id |	Последние до 50 сообщений чата |
| send_msg |	message_id, peer_chat_id, content |	Отправка сообщения. UUID генерирует клиент |
| mark_read |	message_id	| Отметка о прочтении |

## Сообщения сервера - клиенту
| Тип |	Поля |	Назначение |
|---|---|---|
| pow_challenge	| challenge, difficulty_bits	|	Запрос PoW перед обработкой регистрации	|
| auth_ok	| chat_id, token, expires_at	| Успешная авторизация; токен для автологина	|
| auth_failed	| reason	| Отказ (в т.ч. несовместимая версия протокола)	|
| friend_list	| entries: UserBrief[]	| Стартовое состояние: друзья	|
| pending_reqs	| entries: UserBrief[]	| Стартовое состояние: входящие заявки	|
| incoming_req	| from: UserBrief	| Онлайн-уведомление о новой заявке	|
| friend_added	| user: UserBrief	| Заявка принята (обеим сторонам)	|
| user_found \ user_not_found	| user	| Результат поиска	|
| info	| text	| Служебный текстовый ответ	|
| history_msg	| message_id, sender_chat_id, timestamp, content, is_read	| Часть истории; пачка завершается history_end	|
| history_end	| —	| Конец пачки истории	|
| msg_ack	| message_id	| Сообщение сохранено сервером	|
| recv_msg	| message_id, chat_id, sender_chat_id, timestamp, content	| Входящее сообщение	|
| msg_read	| message_id	| Получатель прочитал сообщение (уведомление отправителю)	|
> UserBrief = { chat_id: i64, login: string }.

## Правила сессии
1. Токен выдаётся при каждой успешной авторизации
2. Sliding-продление при каждом использовании
3. Срок жизни настраивается сервером
4. Токен хранится у клиента и передаётся в auth.method.token

## Proof-of-Work
Применяется только к регистрации (антиспам, не целевая защита):

Сервер присылает `pow_challenge`: challenge это 16 hex-символов (`difficulty_bits`)

Клиент ищет nonce: sha256(challenge, nonce_десятичной_строкой) имеет не менее `difficulty_bits` ведущих нулевых бит

Клиент отправляет `pow_solution` { nonce }, сервер проверяет

Сложности сервера: `weak` = 16 бит (~65k попыток), `medium` = 20 бит (~1M), `strong` = 24 бита (~16M).

> Решение вычислительно тяжёлое — выполнять вне async-потока.

## Использование
В файле `Cargo.toml`:
```toml
[dependencies]
Zeevum-protocol = { git = "https://github.com/Zeevum/Zeevum-protocol", tag = "v0.1.0" }
```

В файлах `.rs`
```rs
use Zeevum_protocol::{ClientMsg, encode, decode};

let frame = encode(&ClientMsg::SendMsg {message_id: uuid::Uuid::new_v4(), peer_chat_id: 1234567, content: "Привет".into(),})?; // готовая строка с '\n'
let msg: ClientMsg = decode(&line)?;
```

## Лицензия
[MIT](LICENSE)
