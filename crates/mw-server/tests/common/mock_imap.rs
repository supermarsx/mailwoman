//! Writable IMAP fixture: APPEND bytes survive reconnects and are fetched on sync.
//! Used where test setup must persist mail, rather than inventing POP3 writes.
use base64::Engine as _;
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

#[derive(Clone)]
struct Message {
    raw: Vec<u8>,
    flags: String,
    date: String,
}
pub struct MockImap {
    pub addr: SocketAddr,
    pub messages: Arc<Mutex<Vec<Vec<u8>>>>,
}
impl MockImap {
    pub async fn start(user: &str, password: &str) -> Self {
        Self::start_folder(user, password, false, false).await
    }
    /// Isolated one-message Trash fixture for verified expunge tests.
    pub async fn start_single_trash(user: &str, password: &str) -> Self {
        Self::start_folder(user, password, true, false).await
    }
    /// Isolated single-copy Drafts fixture; replay must not append a second copy.
    pub async fn start_drafts(user: &str, password: &str) -> Self {
        Self::start_folder(user, password, false, true).await
    }
    async fn start_folder(user: &str, password: &str, trash: bool, drafts: bool) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let state = Arc::new(Mutex::new(Vec::<Message>::new()));
        let messages = Arc::new(Mutex::new(Vec::new()));
        let user = user.to_owned();
        let password = password.to_owned();
        let captured = messages.clone();
        tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                let state = state.clone();
                let user = user.clone();
                let password = password.clone();
                let captured = captured.clone();
                tokio::spawn(async move {
                    let _ = serve(stream, state, captured, &user, &password, trash, drafts).await;
                });
            }
        });
        Self { addr, messages }
    }
    pub fn url(&self) -> String {
        format!("imap://{}", self.addr)
    }
}
const CAPS: &str = "IMAP4rev1 UIDPLUS AUTH=PLAIN";
async fn serve(
    stream: TcpStream,
    state: Arc<Mutex<Vec<Message>>>,
    captured: Arc<Mutex<Vec<Vec<u8>>>>,
    user: &str,
    password: &str,
    trash: bool,
    drafts: bool,
) -> std::io::Result<()> {
    let (read, mut write) = stream.into_split();
    let mut reader = BufReader::new(read);
    let mut authenticated = false;
    write
        .write_all(format!("* OK [CAPABILITY {CAPS}] ready\r\n").as_bytes())
        .await?;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).await? == 0 {
            return Ok(());
        }
        let Some((tag, command)) = line.trim_end().split_once(' ') else {
            continue;
        };
        let upper = command.to_ascii_uppercase();
        let mut response = String::new();
        if upper == "CAPABILITY" {
            response = format!("* CAPABILITY {CAPS}\r\n{tag} OK capability\r\n");
        } else if upper.starts_with("AUTHENTICATE PLAIN") {
            write.write_all(b"+\r\n").await?;
            let mut encoded = String::new();
            reader.read_line(&mut encoded).await?;
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(encoded.trim())
                .unwrap_or_default();
            let pieces: Vec<&[u8]> = bytes.split(|byte| *byte == 0).collect();
            authenticated = pieces.len() == 3
                && pieces[1] == user.as_bytes()
                && pieces[2] == password.as_bytes();
            response = format!(
                "{tag} {} authentication\r\n",
                if authenticated { "OK" } else { "NO" }
            );
        } else if !authenticated {
            response = format!("{tag} NO authentication required\r\n");
        } else if upper.starts_with("LIST ") {
            response = if drafts {
                format!("* LIST (\\HasNoChildren \\Drafts) \"/\" \"Drafts\"\r\n{tag} OK list\r\n")
            } else if trash {
                format!("* LIST (\\HasNoChildren \\Trash) \"/\" \"Trash\"\r\n{tag} OK list\r\n")
            } else {
                format!("* LIST (\\HasNoChildren) \"/\" \"INBOX\"\r\n{tag} OK list\r\n")
            };
        } else if upper.starts_with("STATUS ") {
            let count = state.lock().await.len();
            response = format!(
                "* STATUS \"{}\" (MESSAGES {count} UNSEEN {count} UIDNEXT {} UIDVALIDITY 55)\r\n{tag} OK status\r\n",
                if drafts {
                    "Drafts"
                } else if trash {
                    "Trash"
                } else {
                    "INBOX"
                },
                count + 1
            );
        } else if upper.starts_with("SELECT ") || upper.starts_with("EXAMINE ") {
            let count = state.lock().await.len();
            response = format!(
                "* {count} EXISTS\r\n* FLAGS (\\Seen \\Answered \\Flagged \\Deleted \\Draft)\r\n* OK [UIDVALIDITY 55] epoch\r\n* OK [UIDNEXT {}] next\r\n* OK [PERMANENTFLAGS (\\Seen \\Answered \\Flagged \\Deleted \\Draft \\*)] flags\r\n{tag} OK [READ-WRITE] selected\r\n",
                count + 1
            );
        } else if upper.starts_with("APPEND ") {
            let length = command
                .rsplit_once('{')
                .and_then(|(_, tail)| tail.strip_suffix('}'))
                .and_then(|length| length.parse::<usize>().ok())
                .unwrap();
            assert!(length <= 64 * 1024 * 1024);
            write.write_all(b"+ send literal\r\n").await?;
            let mut raw = vec![0; length];
            reader.read_exact(&mut raw).await?;
            let mut ending = [0; 2];
            reader.read_exact(&mut ending).await?;
            assert_eq!(&ending, b"\r\n");
            let quoted: Vec<&str> = command.split('"').collect();
            let date = if quoted.len() >= 5 {
                quoted[quoted.len() - 2].to_string()
            } else {
                "01-Jan-2024 00:00:00 +0000".into()
            };
            let flags = command
                .split_once('(')
                .and_then(|(_, tail)| tail.split_once(')'))
                .map(|(flags, _)| flags.to_string())
                .unwrap_or_default();
            captured.lock().await.push(raw.clone());
            let mut messages = state.lock().await;
            assert!(
                !(trash || drafts) || messages.is_empty(),
                "single-message Trash/Drafts fixture"
            );
            messages.push(Message { raw, flags, date });
            response = format!("{tag} OK [APPENDUID 55 {}] saved\r\n", messages.len());
        } else if upper.starts_with("UID FETCH ") {
            let sequence = command.split_whitespace().nth(2).unwrap();
            let wants_body = upper.contains("BODY");
            let messages = state.lock().await.clone();
            for (index, message) in messages.iter().enumerate() {
                let uid = index + 1;
                if !contains_uid(sequence, uid, messages.len()) {
                    continue;
                }
                if wants_body {
                    write.write_all(format!("* {uid} FETCH (UID {uid} FLAGS ({}) INTERNALDATE \"{}\" BODY[] {{{}}}\r\n",message.flags,message.date,message.raw.len()).as_bytes()).await?;
                    write.write_all(&message.raw).await?;
                    write.write_all(b")\r\n").await?;
                } else {
                    response.push_str(&format!(
                        "* {uid} FETCH (UID {uid} FLAGS ({}))\r\n",
                        message.flags
                    ));
                }
            }
            response.push_str(&format!("{tag} OK fetch\r\n"));
        } else if upper.starts_with("UID SEARCH ") {
            let count = state.lock().await.len();
            response = format!(
                "* SEARCH {}\r\n{tag} OK search\r\n",
                (1..=count)
                    .map(|uid| uid.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        } else if (trash || drafts) && upper.starts_with("UID EXPUNGE ") {
            let mut messages = state.lock().await;
            if messages
                .first()
                .is_some_and(|message| message.flags.contains("\\Deleted"))
            {
                messages.clear();
                captured.lock().await.clear();
                response.push_str("* 1 EXPUNGE\r\n");
            }
            response.push_str(&format!("{tag} OK expunged\r\n"));
        } else if upper.starts_with("UID STORE ") {
            if (trash || drafts) && upper.contains("\\DELETED") {
                for message in state.lock().await.iter_mut() {
                    message.flags.push_str(" \\Deleted");
                }
            }
            response = format!("{tag} OK flags\r\n");
        } else if upper == "NOOP" {
            response = format!("{tag} OK noop\r\n");
        } else if upper == "LOGOUT" {
            write
                .write_all(format!("* BYE closing\r\n{tag} OK logout\r\n").as_bytes())
                .await?;
            return Ok(());
        } else {
            response = format!("{tag} BAD unsupported fixture command\r\n");
        }
        write.write_all(response.as_bytes()).await?;
    }
}
fn contains_uid(sequence: &str, uid: usize, max: usize) -> bool {
    let number = |text: &str| {
        if text == "*" {
            max
        } else {
            text.parse().unwrap_or(0)
        }
    };
    sequence.split(',').any(|part| match part.split_once(':') {
        Some((a, b)) => {
            let a = number(a);
            let b = number(b);
            (a.min(b)..=a.max(b)).contains(&uid)
        }
        None => number(part) == uid,
    })
}
