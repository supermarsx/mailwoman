//! t58 — email the admin console's address.
//!
//! Security > Console address has a "Send by email" action, so an administrator
//! can keep the console's address on record (it is deliberately hard to guess,
//! and a change applies only at the next start). Two routes:
//!
//! * `GET {console}/console-path/mail`: whether outbound system mail can send,
//!   and the addresses the dialog offers (the admin's sign-in name when it is an
//!   address, and the configured operator recipients).
//! * `POST {console}/console-path/mail {recipient, locale?}`: queue the message
//!   for one recipient.
//!
//! Sending needs `security.read` (the same as seeing the address) and the step-up
//! that moving the console needs (`StepUpAction::ConsolePath`). Each send is
//! audited (`admin-console-path-emailed`: the recipient, which the audit redactor
//! masks, its domain, and path fingerprints, never the path) and rate limited per
//! admin and per server (see [`PER_ADMIN_PER_HOUR`]).
//!
//! The message goes through `compliance_mail`, the server's one outbound system
//! mailer (SMTP or JMAP submission, encrypted queue). It is plain text plus an
//! HTML alternative with no remote content, in the console language the client
//! sends (English when that is not a bundled language). Its parts can be replaced
//! with `serverText.admin-console-url-{subject,intro,signin,private}` (global
//! scope, a locale map, like the account notices); `{name}`, `{host}`, `{admin}`
//! and `{when}` are filled in. The address itself is built here, from
//! `ExternalBase` (`MW_PUBLIC_URL`, else a trusted proxy's forwarded host, else
//! `Host`), `MW_BASE_PATH` and the console path; nothing in the request names it.
//!
//! A pending change is described as it behaves: the path in force works until
//! the server restarts, and from then on only the stored path does (there is no
//! alias, and that start ends every admin session).

use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use axum::Json;
use axum::extract::State;
use axum::http::{Extensions, HeaderMap, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use mw_store::settings_policy::SettingSubject;
use serde::Deserialize;
use serde_json::{Value, json};

use super::{STORED_KEY, Source, admin_path, fingerprint, source};
use crate::AppState;
use crate::admin_ctx::{AdminApiError, AdminCtx, AdminJson};
use crate::admin_router::AdminRegistrar;
use crate::capabilities::Capability;
use crate::scope_mw::rate_limit::KeyedRateLimiter;

/// Messages one administrator may queue per hour.
pub const PER_ADMIN_PER_HOUR: u32 = 3;
/// Messages the whole server may queue per hour.
pub const GLOBAL_PER_HOUR: u32 = 20;

/// The operator recipient lists the dialog offers.
const OPERATOR_KEYS: [&str; 3] = [
    "dsr.notify.operatorAddresses",
    "incidents.notify.operatorAddresses",
    "breach.notify.operatorAddresses",
];

pub(super) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::GET,
        "/console-path/mail",
        Capability::SecurityRead,
        status,
    );
    r.admin_endpoint(
        Method::POST,
        "/console-path/mail",
        Capability::SecurityRead,
        send,
    );
}

// ─── recipients ──────────────────────────────────────────────────────────────────

/// One plain mailbox address (`local@host.domain`): no display name, no list,
/// no quoted local part, no address literal.
fn single_recipient(raw: &str) -> Option<String> {
    let addr = raw.trim();
    if addr.is_empty()
        || addr.len() > 254
        || addr
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || ",;<>\"()[]\\".contains(c))
    {
        return None;
    }
    let (local, domain) = addr.rsplit_once('@')?;
    if local.is_empty() || local.contains('@') || !domain.contains('.') {
        return None;
    }
    let labels_ok = domain.split('.').all(|l| {
        !l.is_empty()
            && l.len() <= 63
            && !l.starts_with('-')
            && !l.ends_with('-')
            && l.chars().all(|c| c.is_alphanumeric() || c == '-')
    });
    if !labels_ok {
        return None;
    }
    mw_smtp::validate_mailbox(addr).ok()?;
    Some(addr.to_owned())
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RecipientKind {
    /// The admin's own sign-in name, which is an address.
    SignIn,
    /// A configured operator recipient.
    Operator,
    /// Typed in the dialog.
    Other,
}

impl RecipientKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::SignIn => "signIn",
            Self::Operator => "operator",
            Self::Other => "other",
        }
    }
}

/// The addresses the dialog offers, own sign-in name first, without duplicates.
async fn suggestions(state: &AppState, ctx: &AdminCtx) -> Vec<(String, RecipientKind)> {
    let mut out: Vec<(String, RecipientKind)> = Vec::new();
    let mut push = |addr: &str, kind| {
        if let Some(a) = single_recipient(addr)
            && !out.iter().any(|(b, _)| b.eq_ignore_ascii_case(&a))
            && out.len() < 10
        {
            out.push((a, kind));
        }
    };
    push(&ctx.username, RecipientKind::SignIn);
    let subject = global_subject();
    if let Ok(rows) = crate::settings_registry::effective_all(&state.store, &subject, |d| {
        OPERATOR_KEYS.contains(&d.key)
    })
    .await
    {
        for key in OPERATOR_KEYS {
            let Some(row) = rows.iter().find(|r| r.key == key) else {
                continue;
            };
            for addr in row.value.as_array().into_iter().flatten() {
                if let Some(a) = addr.as_str() {
                    push(a, RecipientKind::Operator);
                }
            }
        }
    }
    out
}

fn global_subject() -> SettingSubject<'static> {
    SettingSubject {
        personal_owner: None,
        domain: "",
        address: "",
        account_id: "",
    }
}

// ─── rate limit ─────────────────────────────────────────────────────────────────

struct Limits {
    admin: KeyedRateLimiter,
    global: KeyedRateLimiter,
}

fn limits() -> std::sync::MutexGuard<'static, Limits> {
    static L: OnceLock<Mutex<Limits>> = OnceLock::new();
    L.get_or_init(|| {
        let per_hour = |n: u32| f64::from(n) / 3600.0;
        Mutex::new(Limits {
            admin: KeyedRateLimiter::new(
                f64::from(PER_ADMIN_PER_HOUR),
                per_hour(PER_ADMIN_PER_HOUR),
                4096,
            ),
            global: KeyedRateLimiter::new(
                f64::from(GLOBAL_PER_HOUR),
                per_hour(GLOBAL_PER_HOUR),
                4096,
            ),
        })
    })
    .lock()
    .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Charge one message to `admin` and to the server. The budgets are per process
/// and per store (`Store::instance_id`; test binaries build several servers),
/// so they reset at restart and each replica has its own.
fn charge(state: &AppState, admin: &str) -> Result<(), AdminApiError> {
    let app = state.store.instance_id();
    let (admin_key, global_key) = (format!("{app}:{admin}"), app.to_string());
    let now = Instant::now();
    let mut l = limits();
    let wait = match l.admin.retry_after_at(&admin_key, now) {
        Some(wait) => Some(wait),
        None => {
            let wait = l.global.retry_after_at(&global_key, now);
            if wait.is_some() {
                l.admin.refund_at(&admin_key, now);
            }
            wait
        }
    };
    match wait {
        None => Ok(()),
        Some(wait) => Err(AdminApiError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "rateLimited",
            "Too many console address emails were sent recently. Try again later.",
        )
        .with("retryAfterSecs", wait.as_secs().clamp(1, 3600))),
    }
}

fn refund(state: &AppState, admin: &str) {
    let app = state.store.instance_id();
    let now = Instant::now();
    let mut l = limits();
    l.admin.refund_at(&format!("{app}:{admin}"), now);
    l.global.refund_at(&app.to_string(), now);
}

// ─── the address ─────────────────────────────────────────────────────────────────

/// What the message says about the console's address.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Addresses {
    /// The address the running server answers at.
    pub current: String,
    /// The address the next start applies, when it differs.
    pub after_restart: Option<String>,
    /// `host[:port]`, for the subject.
    pub host: String,
}

/// `origin` + deploy prefix + console path + `/`.
pub(crate) fn console_url(origin: &str, base: &str, path: &str) -> String {
    format!("{}{base}{path}/", origin.trim_end_matches('/'))
}

fn addresses(
    state: &AppState,
    headers: &HeaderMap,
    ext: &Extensions,
    stored: Option<&str>,
) -> Result<Addresses, AdminApiError> {
    let origin = state.external.base_url(headers, ext).ok_or_else(|| {
        AdminApiError::conflict(
            "publicAddressUnknown",
            "The server cannot tell its public address. Set MW_PUBLIC_URL.",
        )
    })?;
    let host = origin
        .split_once("://")
        .map_or(origin.as_str(), |(_, h)| h)
        .to_owned();
    let path = admin_path();
    let after_restart = stored
        .filter(|s| source() != Source::Env && *s != path)
        .map(|p| console_url(&origin, &state.base_path, p));
    Ok(Addresses {
        current: console_url(&origin, &state.base_path, &path),
        after_restart,
        host,
    })
}

// ─── the message ─────────────────────────────────────────────────────────────────

/// The bundled text of one language.
#[derive(Clone, Copy)]
pub(crate) struct Text {
    pub lang: &'static str,
    pub subject: &'static str,
    pub intro: &'static str,
    pub current: &'static str,
    pub open: &'static str,
    pub pending_label: &'static str,
    pub pending_note: &'static str,
    pub signin: &'static str,
    pub private: &'static str,
}

const fn text(
    lang: &'static str,
    [
        subject,
        intro,
        current,
        open,
        pending_label,
        pending_note,
        signin,
        private,
    ]: [&'static str; 8],
) -> Text {
    Text {
        lang,
        subject,
        intro,
        current,
        open,
        pending_label,
        pending_note,
        signin,
        private,
    }
}

const TEXTS: [Text; 14] = [
    text(
        "en",
        [
            "Admin console address for {name} ({host})",
            "{admin} asked for this message on {when}.",
            "Admin console address:",
            "Open the admin console",
            "Address after the next server restart:",
            "The address above works until the server restarts. After the restart only this new address works, and every admin session ends.",
            "This address opens the sign-in page only. Signing in still needs your administrator credentials and any second factor set up for your account.",
            "Keep this address private: do not forward this message or post the address.",
        ],
    ),
    text(
        "ar",
        [
            "عنوان وحدة تحكم الإدارة لـ {name} ({host})",
            "طلب {admin} هذه الرسالة في {when}.",
            "عنوان وحدة تحكم الإدارة:",
            "فتح وحدة تحكم الإدارة",
            "العنوان بعد إعادة تشغيل الخادم التالية:",
            "يعمل العنوان أعلاه حتى إعادة تشغيل الخادم. بعد إعادة التشغيل يعمل العنوان الجديد فقط، وتنتهي جميع جلسات الإدارة.",
            "يفتح هذا العنوان صفحة تسجيل الدخول فقط. لا يزال تسجيل الدخول يتطلب بيانات اعتماد المسؤول وأي عامل ثانٍ مُعدّ لحسابك.",
            "حافظ على سرية هذا العنوان: لا تُعِد توجيه هذه الرسالة ولا تنشر العنوان.",
        ],
    ),
    text(
        "de",
        [
            "Adresse der Administrationskonsole für {name} ({host})",
            "{admin} hat diese Nachricht am {when} angefordert.",
            "Adresse der Administrationskonsole:",
            "Administrationskonsole öffnen",
            "Adresse nach dem nächsten Serverneustart:",
            "Die obige Adresse funktioniert bis zum Neustart des Servers. Danach funktioniert nur noch die neue Adresse, und alle Admin-Sitzungen werden beendet.",
            "Diese Adresse öffnet nur die Anmeldeseite. Für die Anmeldung sind weiterhin Ihre Administrator-Zugangsdaten und ein für Ihr Konto eingerichteter zweiter Faktor nötig.",
            "Halten Sie diese Adresse vertraulich: Leiten Sie diese Nachricht nicht weiter und veröffentlichen Sie die Adresse nicht.",
        ],
    ),
    text(
        "es",
        [
            "Dirección de la consola de administración de {name} ({host})",
            "{admin} solicitó este mensaje el {when}.",
            "Dirección de la consola de administración:",
            "Abrir la consola de administración",
            "Dirección tras el próximo reinicio del servidor:",
            "La dirección anterior funciona hasta que se reinicie el servidor. Después del reinicio solo funciona la nueva dirección y se cierran todas las sesiones de administración.",
            "Esta dirección solo abre la página de inicio de sesión. Para iniciar sesión siguen siendo necesarias sus credenciales de administrador y el segundo factor que tenga configurado.",
            "Mantenga esta dirección en privado: no reenvíe este mensaje ni publique la dirección.",
        ],
    ),
    text(
        "fr",
        [
            "Adresse de la console d’administration de {name} ({host})",
            "{admin} a demandé ce message le {when}.",
            "Adresse de la console d’administration :",
            "Ouvrir la console d’administration",
            "Adresse après le prochain redémarrage du serveur :",
            "L’adresse ci-dessus fonctionne jusqu’au redémarrage du serveur. Ensuite, seule la nouvelle adresse fonctionne et toutes les sessions d’administration prennent fin.",
            "Cette adresse ouvre uniquement la page de connexion. La connexion exige toujours vos identifiants d’administrateur et le second facteur configuré pour votre compte.",
            "Gardez cette adresse confidentielle : ne transférez pas ce message et ne publiez pas l’adresse.",
        ],
    ),
    text(
        "it",
        [
            "Indirizzo della console di amministrazione di {name} ({host})",
            "{admin} ha richiesto questo messaggio il {when}.",
            "Indirizzo della console di amministrazione:",
            "Apri la console di amministrazione",
            "Indirizzo dopo il prossimo riavvio del server:",
            "L’indirizzo sopra funziona fino al riavvio del server. Dopo il riavvio funziona solo il nuovo indirizzo e tutte le sessioni di amministrazione vengono chiuse.",
            "Questo indirizzo apre solo la pagina di accesso. Per accedere servono comunque le credenziali di amministratore e l’eventuale secondo fattore configurato per l’account.",
            "Mantieni riservato questo indirizzo: non inoltrare questo messaggio e non pubblicare l’indirizzo.",
        ],
    ),
    text(
        "ja",
        [
            "{name} ({host}) の管理コンソールのアドレス",
            "このメッセージは {admin} が {when} に依頼しました。",
            "管理コンソールのアドレス:",
            "管理コンソールを開く",
            "次回のサーバー再起動後のアドレス:",
            "上のアドレスはサーバーが再起動するまで使えます。再起動後は新しいアドレスだけが使え、すべての管理セッションが終了します。",
            "このアドレスで開くのはサインインページだけです。サインインには引き続き管理者の認証情報と、アカウントに設定済みの第 2 要素が必要です。",
            "このアドレスは公開しないでください。このメッセージを転送したり、アドレスを掲載したりしないでください。",
        ],
    ),
    text(
        "nl",
        [
            "Adres van de beheerconsole voor {name} ({host})",
            "{admin} heeft dit bericht aangevraagd op {when}.",
            "Adres van de beheerconsole:",
            "Beheerconsole openen",
            "Adres na de volgende herstart van de server:",
            "Het adres hierboven werkt tot de server opnieuw start. Na de herstart werkt alleen het nieuwe adres en worden alle beheersessies beëindigd.",
            "Dit adres opent alleen de aanmeldpagina. Voor het aanmelden zijn nog steeds uw beheerdersgegevens en een voor uw account ingestelde tweede factor nodig.",
            "Houd dit adres privé: stuur dit bericht niet door en publiceer het adres niet.",
        ],
    ),
    text(
        "pl",
        [
            "Adres konsoli administracyjnej dla {name} ({host})",
            "Wiadomość wysłana na prośbę: {admin}, {when}.",
            "Adres konsoli administracyjnej:",
            "Otwórz konsolę administracyjną",
            "Adres po następnym ponownym uruchomieniu serwera:",
            "Powyższy adres działa do ponownego uruchomienia serwera. Potem działa tylko nowy adres, a wszystkie sesje administratorów zostają zakończone.",
            "Ten adres otwiera tylko stronę logowania. Do zalogowania nadal potrzebne są dane logowania administratora i drugi składnik skonfigurowany dla konta.",
            "Zachowaj ten adres w tajemnicy: nie przekazuj tej wiadomości dalej i nie publikuj adresu.",
        ],
    ),
    text(
        "pt-BR",
        [
            "Endereço do console de administração de {name} ({host})",
            "{admin} solicitou esta mensagem em {when}.",
            "Endereço do console de administração:",
            "Abrir o console de administração",
            "Endereço após a próxima reinicialização do servidor:",
            "O endereço acima funciona até o servidor reiniciar. Depois da reinicialização, só o novo endereço funciona e todas as sessões de administração são encerradas.",
            "Este endereço abre apenas a página de login. Para entrar, ainda são necessárias suas credenciais de administrador e o segundo fator configurado na sua conta.",
            "Mantenha este endereço em sigilo: não encaminhe esta mensagem nem publique o endereço.",
        ],
    ),
    text(
        "pt-PT",
        [
            "Endereço da consola de administração de {name} ({host})",
            "{admin} pediu esta mensagem em {when}.",
            "Endereço da consola de administração:",
            "Abrir a consola de administração",
            "Endereço após o próximo reinício do servidor:",
            "O endereço acima funciona até o servidor reiniciar. Após o reinício, só o novo endereço funciona e todas as sessões de administração terminam.",
            "Este endereço abre apenas a página de início de sessão. Para iniciar sessão continuam a ser necessárias as suas credenciais de administrador e o segundo fator configurado na sua conta.",
            "Mantenha este endereço privado: não reencaminhe esta mensagem nem publique o endereço.",
        ],
    ),
    text(
        "ru",
        [
            "Адрес консоли администратора для {name} ({host})",
            "Это сообщение запрошено учётной записью {admin} {when}.",
            "Адрес консоли администратора:",
            "Открыть консоль администратора",
            "Адрес после следующего перезапуска сервера:",
            "Адрес выше работает до перезапуска сервера. После перезапуска работает только новый адрес, а все сеансы администраторов завершаются.",
            "По этому адресу открывается только страница входа. Для входа по-прежнему нужны учётные данные администратора и второй фактор, настроенный для вашей учётной записи.",
            "Не разглашайте этот адрес: не пересылайте это сообщение и не публикуйте адрес.",
        ],
    ),
    text(
        "uk",
        [
            "Адреса консолі адміністратора для {name} ({host})",
            "Це повідомлення запитано обліковим записом {admin} {when}.",
            "Адреса консолі адміністратора:",
            "Відкрити консоль адміністратора",
            "Адреса після наступного перезапуску сервера:",
            "Адреса вище працює до перезапуску сервера. Після перезапуску працює лише нова адреса, а всі сеанси адміністраторів завершуються.",
            "За цією адресою відкривається лише сторінка входу. Для входу й надалі потрібні облікові дані адміністратора та другий фактор, налаштований для вашого облікового запису.",
            "Тримайте цю адресу в таємниці: не пересилайте це повідомлення й не публікуйте адресу.",
        ],
    ),
    text(
        "zh",
        [
            "{name}（{host}）的管理控制台地址",
            "{admin} 于 {when} 请求了此邮件。",
            "管理控制台地址：",
            "打开管理控制台",
            "下次服务器重启后的地址：",
            "上面的地址在服务器重启前有效。重启后只有新地址有效，所有管理会话都会结束。",
            "此地址只打开登录页面。登录仍需要你的管理员凭据以及账户已设置的第二验证因素。",
            "请对此地址保密：不要转发此邮件，也不要公开此地址。",
        ],
    ),
];

/// The bundled text for a language tag: an exact bundled locale, else its
/// primary subtag (`pt` is Brazilian, as in the web client), else English.
pub(crate) fn text_for(language: Option<&str>) -> &'static Text {
    let tag = language.unwrap_or("en").trim();
    if let Some(t) = TEXTS.iter().find(|t| t.lang.eq_ignore_ascii_case(tag)) {
        return t;
    }
    let primary = tag.split(['-', '_']).next().unwrap_or("en");
    let primary = if primary.eq_ignore_ascii_case("pt") {
        let european = tag
            .split(['-', '_'])
            .nth(1)
            .is_some_and(|r| !r.eq_ignore_ascii_case("BR"));
        if european { "pt-PT" } else { "pt-BR" }
    } else {
        primary
    };
    TEXTS
        .iter()
        .find(|t| t.lang.eq_ignore_ascii_case(primary))
        .unwrap_or(&TEXTS[0])
}

/// The parts an administrator replaced with `serverText.admin-console-url-*`.
#[derive(Default, Clone, Debug)]
pub(crate) struct Overrides {
    pub subject: Option<String>,
    pub intro: Option<String>,
    pub signin: Option<String>,
    pub private: Option<String>,
}

async fn overrides(state: &AppState, lang: &str) -> Overrides {
    let subject = global_subject();
    let mut out = Overrides::default();
    for (part, slot) in [
        ("subject", &mut out.subject),
        ("intro", &mut out.intro),
        ("signin", &mut out.signin),
        ("private", &mut out.private),
    ] {
        let key = format!("serverText.admin-console-url-{part}");
        let Ok(Some(row)) =
            crate::settings_registry::effective_for(&state.store, &key, &subject).await
        else {
            continue;
        };
        if let Some(text) = row.value.get(lang).and_then(Value::as_str) {
            // One line each; control characters become spaces.
            let clean: String = text
                .trim()
                .chars()
                .map(|c| if c.is_control() { ' ' } else { c })
                .collect();
            if !clean.is_empty() {
                *slot = Some(clean);
            }
        }
    }
    out
}

/// What goes into one message.
pub(crate) struct Message<'a> {
    pub text: &'a Text,
    pub overrides: &'a Overrides,
    /// The instance name (branding).
    pub name: &'a str,
    pub admin: &'a str,
    /// `YYYY-MM-DD HH:MM UTC`.
    pub when: &'a str,
    pub addresses: &'a Addresses,
}

/// A rendered message.
pub(crate) struct Rendered {
    pub subject: String,
    pub text: String,
    pub html: String,
}

pub(crate) fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

/// One line of text (the subject is a header; names never break lines).
fn one_line(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}

pub(crate) fn render(m: &Message<'_>) -> Rendered {
    let fill = |template: &str| {
        template
            .replace("{name}", &one_line(m.name))
            .replace("{host}", &m.addresses.host)
            .replace("{admin}", &one_line(m.admin))
            .replace("{when}", m.when)
    };
    let pick = |custom: &Option<String>, bundled: &str| fill(custom.as_deref().unwrap_or(bundled));
    let t = m.text;
    let subject = pick(&m.overrides.subject, t.subject);
    let intro = pick(&m.overrides.intro, t.intro);
    let signin = pick(&m.overrides.signin, t.signin);
    let private = pick(&m.overrides.private, t.private);
    let footer = format!("{} · {}", one_line(m.name), m.addresses.host);

    // Plain text: each address alone on its own line, never wrapped (the body
    // is sent base64-encoded, so nothing reflows it on the way).
    let mut text = format!("{intro}\n\n{}\n{}\n", t.current, m.addresses.current);
    if let Some(next) = &m.addresses.after_restart {
        text.push_str(&format!(
            "\n{}\n{next}\n{}\n",
            t.pending_label, t.pending_note
        ));
    }
    text.push_str(&format!("\n{signin}\n{private}\n\n-- \n{footer}\n"));

    let e = escape_html;
    let dir = if t.lang == "ar" { "rtl" } else { "ltr" };
    let link = |url: &str, label: &str| {
        format!(
            "<p style=\"margin:0 0 8px\"><a href=\"{u}\" style=\"color:#0b57d0;font-weight:600\">{l}</a></p>\
             <p dir=\"ltr\" style=\"margin:0 0 16px;padding:8px 12px;background:#f3f4f6;border-radius:4px;font-family:ui-monospace,Menlo,Consolas,monospace;font-size:14px;word-break:break-all;text-align:left\">{u}</p>",
            u = e(url),
            l = e(label)
        )
    };
    let mut body = format!(
        "<p style=\"margin:0 0 16px;color:#4b5563;font-size:14px\">{}</p>\
         <p style=\"margin:0 0 8px;font-weight:600\">{}</p>{}",
        e(&intro),
        e(t.current),
        link(&m.addresses.current, t.open)
    );
    if let Some(next) = &m.addresses.after_restart {
        body.push_str(&format!(
            "<p style=\"margin:0 0 8px;font-weight:600\">{}</p>{}<p style=\"margin:0 0 16px;font-size:14px\">{}</p>",
            e(t.pending_label),
            link(next, next),
            e(t.pending_note)
        ));
    }
    body.push_str(&format!(
        "<p style=\"margin:0 0 8px;font-size:14px\">{}</p><p style=\"margin:0;font-size:14px\">{}</p>",
        e(&signin),
        e(&private)
    ));
    let html = format!(
        "<!doctype html><html lang=\"{lang}\" dir=\"{dir}\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width\"><title>{title}</title></head>\
         <body style=\"margin:0;padding:24px 12px;background:#f5f5f5;color:#111827;font-family:system-ui,-apple-system,'Segoe UI',Roboto,sans-serif;line-height:1.5\">\
         <div style=\"max-width:560px;margin:0 auto;background:#ffffff;border:1px solid #e5e7eb;border-radius:8px;padding:24px\">{body}</div>\
         <p style=\"max-width:560px;margin:12px auto 0;text-align:center;font-size:12px;color:#6b7280\">{footer}</p>\
         </body></html>",
        lang = t.lang,
        title = e(&subject),
        footer = e(&footer),
    );
    Rendered {
        subject,
        text,
        html,
    }
}

// ─── routes ──────────────────────────────────────────────────────────────────────

async fn status(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SecurityRead)?;
    let unavailable = crate::compliance_mail::unavailable(&state).await;
    let suggestions: Vec<Value> = suggestions(&state, &ctx)
        .await
        .into_iter()
        .map(|(address, kind)| json!({"address": address, "kind": kind.as_str()}))
        .collect();
    Ok((
        [("cache-control", "private, no-store")],
        Json(json!({
            "configured": unavailable.is_none(),
            "reason": unavailable,
            "suggestions": suggestions,
            "perAdminPerHour": PER_ADMIN_PER_HOUR,
        })),
    )
        .into_response())
}

/// The body of `POST {console}/console-path/mail`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SendRequest {
    recipient: String,
    /// The console language (`de`, `pt-BR`, ...). English when absent or unknown.
    #[serde(default)]
    locale: Option<String>,
}

async fn send(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    ext: Extensions,
    AdminJson(body): AdminJson<SendRequest>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SecurityRead)?;
    let recipient = single_recipient(&body.recipient).ok_or_else(|| {
        AdminApiError::invalid(
            "invalidRecipient",
            "Enter one email address, without a name or a second address.",
            "/recipient",
        )
    })?;
    if let Some(reason) = crate::compliance_mail::unavailable(&state).await {
        return Err(AdminApiError::conflict(
            "mailNotConfigured",
            "Outbound system email is not set up. Configure it in Compliance > Email notifications.",
        )
        .with("reason", reason));
    }
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::ConsolePath).await?;
    let stored = state
        .store
        .get_setting(STORED_KEY)
        .await
        .map_err(AdminApiError::internal)?;
    let addresses = addresses(&state, &headers, &ext, stored.as_deref())?;
    charge(&state, &ctx.username)?;

    let text = text_for(body.locale.as_deref());
    let overrides = overrides(&state, text.lang).await;
    let name = match state.store.branding_snapshot().await {
        Ok(s) => crate::branding_routes::view(&s)["brandName"]
            .as_str()
            .unwrap_or("Mailwoman")
            .to_owned(),
        Err(_) => "Mailwoman".to_owned(),
    };
    let admin = ctx
        .display_name
        .as_deref()
        .filter(|d| !d.trim().is_empty())
        .map_or_else(
            || ctx.username.clone(),
            |d| format!("{d} ({})", ctx.username),
        );
    let when = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();
    let rendered = render(&Message {
        text,
        overrides: &overrides,
        name: &name,
        admin: &admin,
        when: &when,
        addresses: &addresses,
    });
    let event = format!("admin-console-url:{}", uuid::Uuid::new_v4());
    if let Err(e) = crate::compliance_mail::enqueue_with_html(
        &state,
        &event,
        &recipient,
        &rendered.subject,
        &rendered.text,
        Some(&rendered.html),
    )
    .await
    {
        refund(&state, &ctx.username);
        tracing::warn!("console address email: could not queue: {e}");
        return Err(AdminApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "mailQueueFailed",
            "The message could not be queued. Check the outbound email settings.",
        ));
    }
    crate::compliance_mail::deliver_soon(&state);

    let kind = suggestions(&state, &ctx)
        .await
        .into_iter()
        .find(|(a, _)| a.eq_ignore_ascii_case(&recipient))
        .map_or(RecipientKind::Other, |(_, k)| k);
    let domain = recipient
        .rsplit_once('@')
        .map_or("", |(_, d)| d)
        .to_ascii_lowercase();
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "admin-console-path-emailed",
        "admin:console-path",
        None,
        Some(json!({
            "recipient": recipient,
            "recipientDomain": domain,
            "recipientKind": kind.as_str(),
            "path": fingerprint(&admin_path()),
            "afterRestart": addresses.after_restart.as_ref().and(stored.as_deref()).map(fingerprint),
            "locale": text.lang,
            "delivery": event,
        })),
    )
    .await;
    Ok((
        StatusCode::ACCEPTED,
        [("cache-control", "private, no-store")],
        Json(json!({"queued": true, "recipient": recipient, "afterRestart": addresses.after_restart.is_some()})),
    )
        .into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(after: bool) -> Addresses {
        Addresses {
            current: console_url("https://mail.example.org", "/mail", "/ops-7f3k2qwerty"),
            after_restart: after
                .then(|| console_url("https://mail.example.org", "/mail", "/next-9x8c7v6b5n")),
            host: "mail.example.org".into(),
        }
    }

    fn rendered(lang: &str, name: &str, after: bool) -> Rendered {
        let a = sample(after);
        render(&Message {
            text: text_for(Some(lang)),
            overrides: &Overrides::default(),
            name,
            admin: "Root <root>",
            when: "2026-11-24 10:00 UTC",
            addresses: &a,
        })
    }

    #[test]
    fn the_address_keeps_the_deploy_prefix_and_ends_in_a_slash() {
        assert_eq!(
            console_url("https://mail.example.org/", "/mail", "/ops-1"),
            "https://mail.example.org/mail/ops-1/"
        );
        assert_eq!(
            console_url("http://h:8080", "", "/admin"),
            "http://h:8080/admin/"
        );
    }

    #[test]
    fn plain_text_has_each_address_alone_on_a_line() {
        let r = rendered("en", "Example Mail", true);
        let lines: Vec<&str> = r.text.lines().collect();
        assert!(
            lines.contains(&"https://mail.example.org/mail/ops-7f3k2qwerty/"),
            "{}",
            r.text
        );
        assert!(
            lines.contains(&"https://mail.example.org/mail/next-9x8c7v6b5n/"),
            "{}",
            r.text
        );
        assert!(r.text.contains("until the server restarts"));
        assert!(r.text.contains("sign-in page only"));
        assert!(r.text.contains("Keep this address private"));
        assert!(r.text.contains("2026-11-24 10:00 UTC"));
        assert_eq!(
            r.subject,
            "Admin console address for Example Mail (mail.example.org)"
        );
        let single = rendered("en", "Example Mail", false);
        assert!(!single.text.contains("next-9x8c7v6b5n"));
        assert!(!single.text.contains("restart"));
    }

    #[test]
    fn html_escapes_the_instance_name_and_requester_and_links_the_address() {
        let r = rendered("en", "<b>Evil</b> & \"co\"", false);
        assert!(!r.html.contains("<b>Evil</b>"), "{}", r.html);
        assert!(
            r.html
                .contains("&lt;b&gt;Evil&lt;/b&gt; &amp; &quot;co&quot;")
        );
        assert!(r.html.contains("Root &lt;root&gt;"));
        assert!(
            r.html
                .contains("<a href=\"https://mail.example.org/mail/ops-7f3k2qwerty/\"")
        );
        // No remote content: no images, scripts, styles sheets or tracking.
        for banned in ["<img", "<script", "<link", "url(", "src="] {
            assert!(!r.html.contains(banned), "{banned}");
        }
    }

    #[test]
    fn every_language_has_the_placeholders_and_unknown_ones_fall_back() {
        for t in &TEXTS {
            assert!(
                t.subject.contains("{name}") && t.subject.contains("{host}"),
                "{}",
                t.lang
            );
            assert!(
                t.intro.contains("{admin}") && t.intro.contains("{when}"),
                "{}",
                t.lang
            );
        }
        assert_eq!(text_for(Some("de-AT")).lang, "de");
        assert_eq!(text_for(Some("pt")).lang, "pt-BR");
        assert_eq!(text_for(Some("pt-AO")).lang, "pt-PT");
        assert_eq!(text_for(Some("PT-br")).lang, "pt-BR");
        assert_eq!(text_for(Some("xx")).lang, "en");
        assert_eq!(text_for(None).lang, "en");
        assert!(rendered("ar", "N", false).html.contains("dir=\"rtl\""));
        assert!(
            rendered("de", "N", false)
                .subject
                .starts_with("Adresse der")
        );
    }

    #[test]
    fn overrides_replace_their_part_and_fill_placeholders() {
        let a = sample(false);
        let o = Overrides {
            subject: Some("Console for {host}".into()),
            ..Overrides::default()
        };
        let r = render(&Message {
            text: text_for(Some("en")),
            overrides: &o,
            name: "N",
            admin: "root",
            when: "w",
            addresses: &a,
        });
        assert_eq!(r.subject, "Console for mail.example.org");
        assert!(r.text.contains("root asked for this message on w."));
    }

    #[test]
    fn only_one_plain_address_is_a_recipient() {
        for ok in ["ops@example.org", " ops.team+x@mail.example.org "] {
            assert!(single_recipient(ok).is_some(), "{ok}");
        }
        for bad in [
            "",
            "root",
            "a@b",
            "a@example.org, b@example.org",
            "a@example.org;b@example.org",
            "Ops <ops@example.org>",
            "\"a b\"@example.org",
            "a@[127.0.0.1]",
            "a@exa mple.org",
            "a@example.org\r\nBcc: x@example.org",
            "a@-example.org",
        ] {
            assert!(single_recipient(bad).is_none(), "{bad:?}");
        }
    }
}
