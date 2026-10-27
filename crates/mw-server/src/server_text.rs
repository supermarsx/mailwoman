//! Server-generated account notices. Locale selection follows scoped ui.language.
//! Bundled notices cover the web locale catalogue; unknown locales fall back to en.
use crate::{AppState, alerts::AlertKind};

pub(crate) struct Notice {
    pub locale: &'static str,
    pub subject: &'static str,
    pub description: &'static str,
    pub guidance: &'static str,
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
    let subject = mw_store::settings_policy::SettingSubject { personal_owner: None,
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
    notice(&language, kind)
}
pub(crate) fn notice(language: &str, kind: AlertKind) -> Notice {
    let key = language
        .split(['-', '_'])
        .next()
        .unwrap_or("en")
        .to_ascii_lowercase();
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
        "pt" => (
            "pt",
            "Aviso de segurança Mailwoman",
            [
                "A sua conta foi acedida a partir de um novo dispositivo.",
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
                "Your account was signed in to from a new device.",
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
        subject,
        description: descriptions[index],
        guidance,
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
            "ar", "de", "en", "es", "fr", "it", "ja", "nl", "pl", "pt-BR", "ru", "uk", "zh",
        ] {
            assert_eq!(
                notice(language, AlertKind::NewDevice).locale,
                language.split('-').next().unwrap()
            );
            for kind in AlertKind::ALL {
                let text = notice(language, kind);
                assert!(!text.description.is_empty());
                assert!(!text.subject.contains(['\r', '\n']));
            }
        }
    }
}
