//! Server-generated account notices. Locale selection follows scoped ui.language.
//! Bundled notices cover the web locale catalogue; unknown locales fall back to en.
use crate::{AppState, alerts::AlertKind};

use std::borrow::Cow;

pub(crate) struct Notice {
    pub locale: &'static str,
    pub subject: Cow<'static, str>,
    pub description: Cow<'static, str>,
    pub guidance: Cow<'static, str>,
}
pub(crate) async fn account_notice(state: &AppState, account: &str, kind: AlertKind) -> Notice {
    let address = state
        .store
        .security_account_address(account)
        .await
        .ok()
        .flatten()
        .unwrap_or_default();
    let domain = address.rsplit_once('@').map_or("", |(_, domain)| domain);
    let subject = mw_store::settings_policy::SettingSubject {
        personal_owner: None,
        account_id: account,
        address: &address,
        domain,
    };
    let language = crate::settings_registry::effective_for(&state.store, "ui.language", &subject)
        .await
        .ok()
        .flatten()
        .and_then(|v| v.value.as_str().map(str::to_owned))
        .unwrap_or_else(|| "en".into());
    let mut text = notice(&language, kind);
    apply_overrides(&state.store, &subject, kind, &mut text).await;
    text
}

/// The id slug in `serverText.alert-<slug>-{subject,description,guidance}`.
fn kind_slug(kind: AlertKind) -> &'static str {
    match kind {
        AlertKind::NewDevice => "new-device",
        AlertKind::TwoFactorChanged => "two-factor-changed",
        AlertKind::DelegateAdded => "delegate-added",
        AlertKind::ForwardingAdded => "forwarding-added",
        AlertKind::ApiKeyCreated => "api-key-created",
        AlertKind::NewCountry => "new-country",
    }
}

/// Replace a part of the bundled notice with the text an administrator saved
/// for the recipient's domain (or globally) in the notice's own locale. A part
/// with no saved text for that locale keeps the bundled text. Any read error
/// leaves the bundled text, so an alert is never lost to a settings problem.
async fn apply_overrides(
    store: &mw_store::Store,
    subject: &mw_store::settings_policy::SettingSubject<'_>,
    kind: AlertKind,
    text: &mut Notice,
) {
    for part in ["subject", "description", "guidance"] {
        let key = format!("serverText.alert-{}-{part}", kind_slug(kind));
        let Some(value) = crate::settings_registry::effective_for(store, &key, subject)
            .await
            .ok()
            .flatten()
        else {
            continue;
        };
        let Some(custom) = override_text(&value.value, text.locale, part != "subject") else {
            continue;
        };
        let slot = match part {
            "subject" => &mut text.subject,
            "description" => &mut text.description,
            _ => &mut text.guidance,
        };
        *slot = Cow::Owned(custom);
    }
}

/// The non-blank text for `locale` in a saved locale map. Control characters
/// become spaces; line breaks stay only where `multiline` (the subject is a
/// header).
fn override_text(map: &serde_json::Value, locale: &str, multiline: bool) -> Option<String> {
    let text = map.get(locale)?.as_str()?.trim();
    if text.is_empty() {
        return None;
    }
    Some(
        text.chars()
            .map(|c| if c.is_control() && !(multiline && c == '\n') { ' ' } else { c })
            .collect(),
    )
}
/// The match key for a language tag: the lower-cased primary subtag, except
/// that Portuguese is split by region the way the web client negotiates it
/// (`apps/web/src/i18n/locales.ts`). `pt-BR` and a bare `pt` (which CLDR's
/// likely subtags expand to `pt-Latn-BR`) are Brazilian, and every other region
/// (`pt-PT`, `pt-AO`, `pt-MZ` and so on, the regions CLDR parents to `pt-PT`)
/// is European.
fn language_key(language: &str) -> String {
    let mut parts = language.split(['-', '_']);
    let base = parts.next().unwrap_or("en").to_ascii_lowercase();
    if base != "pt" {
        return base;
    }
    let region = parts.find(|p| {
        (p.len() == 2 && p.chars().all(|c| c.is_ascii_alphabetic()))
            || (p.len() == 3 && p.chars().all(|c| c.is_ascii_digit()))
    });
    match region {
        Some(r) if !r.eq_ignore_ascii_case("BR") => "pt-pt".into(),
        _ => "pt-br".into(),
    }
}
pub(crate) fn notice(language: &str, kind: AlertKind) -> Notice {
    let key = language_key(language);
    let index = match kind {
        AlertKind::NewDevice => 0,
        AlertKind::TwoFactorChanged => 1,
        AlertKind::DelegateAdded => 2,
        AlertKind::ForwardingAdded => 3,
        AlertKind::ApiKeyCreated => 4,
        AlertKind::NewCountry => 5,
    };
    let (locale, subject, descriptions, guidance) = match key.as_str() {
        "ar" => (
            "ar",
            "إشعار أمان حساب Mailwoman",
            [
                "تم تسجيل الدخول إلى حسابك من جهاز جديد.",
                "تم تغيير عوامل المصادقة لحسابك.",
                "تم منح صلاحية مفوّضة للوصول إلى صندوق بريدك.",
                "تمت إضافة قاعدة لإعادة توجيه البريد.",
                "تم إنشاء مفتاح تطبيق لحسابك.",
                "تم تسجيل الدخول إلى حسابك من بلد جديد.",
            ],
            "راجع النشاط الأخير والجلسات النشطة في الإعدادات ثم مركز الأمان. إذا لم تتعرف على هذا التغيير، فألغِ الجلسات غير المعروفة واتصل بالمسؤول.",
        ),
        "it" => (
            "it",
            "Avviso di sicurezza Mailwoman",
            [
                "È stato effettuato un accesso al tuo account da un nuovo dispositivo.",
                "I fattori di autenticazione del tuo account sono stati modificati.",
                "È stato concesso un accesso delegato alla tua casella di posta.",
                "È stata aggiunta una regola di inoltro alla tua casella di posta.",
                "È stata creata una chiave applicazione per il tuo account.",
                "È stato effettuato un accesso al tuo account da un nuovo paese.",
            ],
            "Controlla le attività recenti e le sessioni in Impostazioni > Centro sicurezza. Se non riconosci questa modifica, revoca le sessioni sconosciute e contatta il tuo amministratore.",
        ),
        "ja" => (
            "ja",
            "Mailwoman アカウントのセキュリティ通知",
            [
                "新しいデバイスからアカウントにログインしました。",
                "アカウントの認証要素が変更されました。",
                "メールボックスへの委任アクセスが許可されました。",
                "メールボックスに転送ルールが追加されました。",
                "アカウントのアプリケーションキーが作成されました。",
                "新しい国からアカウントにログインしました。",
            ],
            "設定のセキュリティセンターで最近のアクティビティとセッションを確認してください。心当たりのない変更の場合は、不明なセッションを無効にして管理者に連絡してください。",
        ),
        "nl" => (
            "nl",
            "Beveiligingsmelding van Mailwoman",
            [
                "Er is op uw account ingelogd vanaf een nieuw apparaat.",
                "De authenticatiefactoren van uw account zijn gewijzigd.",
                "Er is gedelegeerde toegang tot uw mailbox verleend.",
                "Er is een doorstuurregel aan uw mailbox toegevoegd.",
                "Er is een toepassingssleutel voor uw account gemaakt.",
                "Er is op uw account ingelogd vanuit een nieuw land.",
            ],
            "Bekijk recente activiteit en sessies onder Instellingen > Beveiligingscentrum. Herkent u deze wijziging niet, trek dan onbekende sessies in en neem contact op met uw beheerder.",
        ),
        "pl" => (
            "pl",
            "Powiadomienie o bezpieczeństwie Mailwoman",
            [
                "Zalogowano się na Twoje konto z nowego urządzenia.",
                "Zmieniono czynniki uwierzytelniania Twojego konta.",
                "Przyznano delegowany dostęp do Twojej skrzynki pocztowej.",
                "Dodano regułę przekazywania wiadomości w Twojej skrzynce.",
                "Utworzono klucz aplikacji dla Twojego konta.",
                "Zalogowano się na Twoje konto z nowego kraju.",
            ],
            "Sprawdź ostatnią aktywność i sesje w Ustawieniach > Centrum bezpieczeństwa. Jeśli nie rozpoznajesz tej zmiany, unieważnij nieznane sesje i skontaktuj się z administratorem.",
        ),
        "ru" => (
            "ru",
            "Уведомление безопасности Mailwoman",
            [
                "Выполнен вход в вашу учётную запись с нового устройства.",
                "Факторы аутентификации вашей учётной записи изменены.",
                "Предоставлен делегированный доступ к вашему почтовому ящику.",
                "В ваш почтовый ящик добавлено правило пересылки.",
                "Для вашей учётной записи создан ключ приложения.",
                "Выполнен вход в вашу учётную запись из новой страны.",
            ],
            "Проверьте недавние действия и сеансы в разделе «Настройки > Центр безопасности». Если вы не узнаёте это изменение, отзовите неизвестные сеансы и обратитесь к администратору.",
        ),
        "uk" => (
            "uk",
            "Сповіщення безпеки Mailwoman",
            [
                "Виконано вхід до вашого облікового запису з нового пристрою.",
                "Змінено фактори автентифікації вашого облікового запису.",
                "Надано делегований доступ до вашої поштової скриньки.",
                "До вашої поштової скриньки додано правило пересилання.",
                "Для вашого облікового запису створено ключ застосунку.",
                "Виконано вхід до вашого облікового запису з нової країни.",
            ],
            "Перевірте останні дії та сеанси в розділі «Налаштування > Центр безпеки». Якщо ви не впізнаєте цю зміну, відкличте невідомі сеанси та зверніться до адміністратора.",
        ),
        "zh" => (
            "zh",
            "Mailwoman 账户安全通知",
            [
                "您的账户已从新设备登录。",
                "您的账户身份验证因素已更改。",
                "已授予对您邮箱的委托访问权限。",
                "您的邮箱已添加转发规则。",
                "已为您的账户创建应用密钥。",
                "您的账户已从新的国家或地区登录。",
            ],
            "请在“设置 > 安全中心”中查看最近的活动和会话。如果您不认识此更改，请撤销未知会话并联系管理员。",
        ),
        "es" => (
            "es",
            "Aviso de seguridad de Mailwoman",
            [
                "Se ha iniciado sesión en tu cuenta desde un dispositivo nuevo.",
                "Han cambiado los factores de autenticación de tu cuenta.",
                "Se ha concedido acceso delegado a tu buzón.",
                "Se ha añadido una regla de reenvío a tu buzón.",
                "Se ha creado una clave de aplicación para tu cuenta.",
                "Se ha iniciado sesión en tu cuenta desde un país nuevo.",
            ],
            "Revisa la actividad reciente y las sesiones en Ajustes > Centro de seguridad. Si no reconoces este cambio, revoca las sesiones desconocidas y contacta con tu administrador.",
        ),
        "fr" => (
            "fr",
            "Avis de sécurité Mailwoman",
            [
                "Une connexion à votre compte a eu lieu depuis un nouvel appareil.",
                "Les facteurs d'authentification de votre compte ont changé.",
                "Un accès délégué à votre boîte aux lettres a été accordé.",
                "Une règle de transfert a été ajoutée à votre boîte aux lettres.",
                "Une clé d'application a été créée pour votre compte.",
                "Une connexion à votre compte a eu lieu depuis un nouveau pays.",
            ],
            "Consultez l'activité récente et les sessions dans Paramètres > Centre de sécurité. Si vous ne reconnaissez pas ce changement, révoquez les sessions inconnues et contactez votre administrateur.",
        ),
        "de" => (
            "de",
            "Mailwoman-Sicherheitshinweis",
            [
                "Ihr Konto wurde auf einem neuen Gerät angemeldet.",
                "Die Authentifizierungsfaktoren Ihres Kontos wurden geändert.",
                "Ein delegierter Zugriff auf Ihr Postfach wurde gewährt.",
                "Eine Weiterleitungsregel wurde Ihrem Postfach hinzugefügt.",
                "Ein Anwendungsschlüssel wurde für Ihr Konto erstellt.",
                "Ihr Konto wurde aus einem neuen Land angemeldet.",
            ],
            "Prüfen Sie die letzten Aktivitäten und Sitzungen unter Einstellungen > Sicherheitscenter. Falls Sie diese Änderung nicht erkennen, widerrufen Sie unbekannte Sitzungen und kontaktieren Sie Ihre Administration.",
        ),
        "pt-br" => (
            "pt-BR",
            "Aviso de segurança do Mailwoman",
            [
                "Sua conta foi acessada de um novo navegador ou dispositivo.",
                "Os fatores de autenticação da sua conta foram alterados.",
                "Foi concedido acesso delegado à sua caixa de correio.",
                "Foi adicionada uma regra de encaminhamento à sua caixa de correio.",
                "Foi criada uma chave de aplicativo para sua conta.",
                "Sua conta foi acessada de um novo país.",
            ],
            "Confira a atividade recente e as sessões em Configurações > Central de segurança. Se você não reconhecer esta alteração, revogue as sessões desconhecidas e entre em contato com o administrador.",
        ),
        "pt-pt" => (
            "pt-PT",
            "Aviso de segurança do Mailwoman",
            [
                "A sua conta foi acedida a partir de um novo navegador ou dispositivo.",
                "Os fatores de autenticação da sua conta foram alterados.",
                "Foi concedido acesso delegado à sua caixa de correio.",
                "Foi adicionada uma regra de reencaminhamento à sua caixa de correio.",
                "Foi criada uma chave de aplicação para a sua conta.",
                "A sua conta foi acedida a partir de um novo país.",
            ],
            "Consulte a atividade recente e as sessões em Definições > Centro de segurança. Se não reconhecer esta alteração, revogue as sessões desconhecidas e contacte o administrador.",
        ),
        _ => (
            "en",
            "Mailwoman account security notice",
            [
                "Your account was signed in to from a new browser or device.",
                "Your account's authentication factors changed.",
                "A delegate was granted access to your mailbox.",
                "A forwarding rule was added to your mailbox.",
                "An application key was created for your account.",
                "Your account was signed in to from a new country.",
            ],
            "Review recent activity and active sessions in Settings > Security centre. If you did not make this change, revoke unfamiliar sessions and contact your administrator.",
        ),
    };
    Notice {
        locale,
        subject: subject.into(),
        description: descriptions[index].into(),
        guidance: guidance.into(),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn regional_language_selects_translation_and_unknown_locale_falls_back() {
        let spanish = notice("es-MX", AlertKind::ApiKeyCreated);
        assert_eq!(spanish.locale, "es");
        assert!(spanish.description.contains("clave"));
        assert_eq!(notice("not-a-locale", AlertKind::NewDevice).locale, "en");
        for language in [
            "ar", "de", "en", "es", "fr", "it", "ja", "nl", "pl", "pt-BR", "pt-PT", "ru", "uk",
            "zh",
        ] {
            let expected = if language.starts_with("pt-") {
                language
            } else {
                language.split('-').next().unwrap()
            };
            assert_eq!(notice(language, AlertKind::NewDevice).locale, expected);
            for kind in AlertKind::ALL {
                let text = notice(language, kind);
                assert!(!text.description.is_empty());
                assert!(!text.subject.contains(['\r', '\n']));
            }
        }
    }
    #[test]
    fn portuguese_splits_by_region_like_the_web_client() {
        for tag in [
            "pt-PT",
            "pt_PT",
            "pt-pt",
            "pt-AO",
            "pt-MZ",
            "pt-CV",
            "pt-Latn-AO",
        ] {
            let text = notice(tag, AlertKind::NewDevice);
            assert_eq!(text.locale, "pt-PT", "{tag}");
            assert!(text.description.contains("acedida"), "{tag}");
            assert!(text.guidance.contains("Definições"), "{tag}");
        }
        for tag in ["pt-BR", "pt_BR", "pt", "pt-Latn-BR"] {
            let text = notice(tag, AlertKind::NewDevice);
            assert_eq!(text.locale, "pt-BR", "{tag}");
            assert!(text.description.contains("acessada"), "{tag}");
            assert!(text.guidance.contains("Configurações"), "{tag}");
        }
        assert_ne!(
            notice("pt-PT", AlertKind::NewDevice).guidance,
            notice("pt-BR", AlertKind::NewDevice).guidance
        );
    }
    #[test]
    fn override_text_picks_the_locale_and_cleans_control_characters() {
        let map = serde_json::json!({"en":"  Hello\r\nthere ","fr":"   ","de":7});
        assert_eq!(override_text(&map, "en", false).as_deref(), Some("Hello  there"));
        assert_eq!(override_text(&map, "en", true).as_deref(), Some("Hello \nthere"));
        assert_eq!(override_text(&map, "fr", true), None);
        assert_eq!(override_text(&map, "de", true), None);
        assert_eq!(override_text(&map, "es", true), None);
    }

    fn subject(domain: &str) -> mw_store::settings_policy::SettingSubject<'_> {
        mw_store::settings_policy::SettingSubject {
            personal_owner: None,
            account_id: "",
            address: "",
            domain,
        }
    }

    async fn save(store: &mw_store::Store, domain: &str, key: &str, value: serde_json::Value) {
        store
            .put_setting_value(&mw_store::settings_policy::SettingValueRow {
                scope: mw_store::settings_policy::SettingScope::Domain,
                scope_id: domain.into(),
                key: key.into(),
                value: value.to_string(),
                locked: false,
                updated_by: "test".into(),
                updated_at: 1,
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn a_domain_override_applies_to_that_domain_in_that_locale_only() {
        let store = mw_store::Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        save(&store, "a.example", "serverText.alert-new-device-subject", serde_json::json!({"en":"Sign-in from a new device","fr":"Nouvel appareil"})).await;
        save(&store, "a.example", "serverText.alert-new-device-guidance", serde_json::json!({"en":"Call the help desk.
Ext 42"})).await;

        // The matching domain and locale: subject and guidance replaced, description kept.
        let bundled = notice("en", AlertKind::NewDevice);
        let mut text = notice("en", AlertKind::NewDevice);
        apply_overrides(&store, &subject("a.example"), AlertKind::NewDevice, &mut text).await;
        assert_eq!(text.subject, "Sign-in from a new device");
        assert_eq!(text.guidance, "Call the help desk.
Ext 42");
        assert_eq!(text.description, bundled.description);

        // Another locale with no saved text keeps the bundled text.
        let mut de = notice("de", AlertKind::NewDevice);
        let bundled_de = notice("de", AlertKind::NewDevice);
        apply_overrides(&store, &subject("a.example"), AlertKind::NewDevice, &mut de).await;
        assert_eq!(de.subject, bundled_de.subject);

        // Another domain is untouched.
        let mut other = notice("en", AlertKind::NewDevice);
        apply_overrides(&store, &subject("b.example"), AlertKind::NewDevice, &mut other).await;
        assert_eq!(other.subject, bundled.subject);
        assert_eq!(other.guidance, bundled.guidance);

        // Another alert kind in the same domain is untouched.
        let mut key = notice("en", AlertKind::ApiKeyCreated);
        let bundled_key = notice("en", AlertKind::ApiKeyCreated);
        apply_overrides(&store, &subject("a.example"), AlertKind::ApiKeyCreated, &mut key).await;
        assert_eq!(key.subject, bundled_key.subject);
    }

    #[test]
    fn every_alert_slug_is_a_registered_server_text_key() {
        for kind in AlertKind::ALL {
            for part in ["subject", "description", "guidance"] {
                let key = format!("serverText.alert-{}-{part}", kind_slug(kind));
                assert!(crate::settings_registry::lookup(&key).is_some(), "{key}");
            }
        }
    }
}
