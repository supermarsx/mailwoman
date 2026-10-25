//! Bounded internal login diagnostics. Public authentication responses stay generic.
use mw_engine::backend::EngineError;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginFailureReason {
    NotConfigured,
    Policy,
    Disabled,
    Auth,
    Protocol,
    Unsupported,
    Transport,
    Dns,
    Connect,
    Timeout,
    Certificate,
    Tls,
    Storage,
    Sync,
}
impl LoginFailureReason {
    pub fn code(self) -> &'static str {
        match self {
            Self::NotConfigured => "not-configured",
            Self::Policy => "upstream-policy",
            Self::Disabled => "account-disabled",
            Self::Auth => "auth-rejected",
            Self::Protocol => "protocol",
            Self::Unsupported => "unsupported",
            Self::Transport => "transport",
            Self::Dns => "dns",
            Self::Connect => "connect",
            Self::Timeout => "timeout",
            Self::Certificate => "certificate",
            Self::Tls => "tls",
            Self::Storage => "storage",
            Self::Sync => "sync",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoginFailure {
    pub reason: LoginFailureReason,
}
impl LoginFailure {
    pub fn new(reason: LoginFailureReason) -> Self {
        Self { reason }
    }
    pub fn from_io(error: &io::Error) -> Self {
        if let Some(stage) = error
            .get_ref()
            .and_then(|inner| inner.downcast_ref::<crate::domain_transport::DialFailure>())
        {
            return Self::new(match stage.phase {
                crate::domain_transport::DialPhase::Dns => LoginFailureReason::Dns,
                crate::domain_transport::DialPhase::Connect => LoginFailureReason::Connect,
            });
        }
        let mut source: Option<&(dyn std::error::Error + 'static)> = error
            .get_ref()
            .map(|inner| inner as &(dyn std::error::Error + 'static))
            .or(Some(error));
        for _ in 0..8 {
            let Some(current) = source else {
                break;
            };
            if let Some(tls) = current.downcast_ref::<rustls::Error>() {
                return Self::new(if matches!(tls, rustls::Error::InvalidCertificate(_)) {
                    LoginFailureReason::Certificate
                } else {
                    LoginFailureReason::Tls
                });
            }
            source = current.source();
        }
        Self::new(match error.kind() {
            io::ErrorKind::TimedOut => LoginFailureReason::Timeout,
            io::ErrorKind::PermissionDenied => LoginFailureReason::Policy,
            io::ErrorKind::ConnectionRefused
            | io::ErrorKind::ConnectionReset
            | io::ErrorKind::NotConnected
            | io::ErrorKind::ConnectionAborted => LoginFailureReason::Connect,
            io::ErrorKind::Unsupported => LoginFailureReason::Unsupported,
            io::ErrorKind::InvalidData => LoginFailureReason::Protocol,
            _ => LoginFailureReason::Transport,
        })
    }
    pub fn from_backend(error: EngineError, observed: Option<LoginFailureReason>) -> Self {
        Self::new(match error {
            EngineError::Auth(_) => LoginFailureReason::Auth,
            EngineError::Protocol(_) | EngineError::MailboxNotFound(_) => {
                LoginFailureReason::Protocol
            }
            EngineError::Unsupported(_) => LoginFailureReason::Unsupported,
            EngineError::Transport(_) => observed.unwrap_or(LoginFailureReason::Transport),
            _ => LoginFailureReason::Storage,
        })
    }
}
impl std::fmt::Display for LoginFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.reason.code())
    }
}
impl std::error::Error for LoginFailure {}
impl From<String> for LoginFailure {
    fn from(_: String) -> Self {
        Self::new(LoginFailureReason::NotConfigured)
    }
}
impl From<&str> for LoginFailure {
    fn from(_: &str) -> Self {
        Self::new(LoginFailureReason::NotConfigured)
    }
}
impl From<LoginFailure> for String {
    fn from(value: LoginFailure) -> Self {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn typed_backend_failures_never_use_server_text_as_the_reason() {
        assert_eq!(
            LoginFailure::from_backend(EngineError::Auth("sensitive server reply".into()), None)
                .to_string(),
            "auth-rejected"
        );
        assert_eq!(
            LoginFailure::from_backend(
                EngineError::Transport("certificate words are not evidence".into()),
                None
            )
            .to_string(),
            "transport"
        );
        assert_eq!(
            LoginFailure::from_backend(
                EngineError::Transport("anything".into()),
                Some(LoginFailureReason::Dns)
            )
            .to_string(),
            "dns"
        );
        assert_eq!(
            LoginFailure::from_io(&io::Error::new(io::ErrorKind::TimedOut, "peer details"))
                .to_string(),
            "timeout"
        );
        assert_eq!(
            LoginFailure::from_io(&io::Error::new(
                io::ErrorKind::InvalidData,
                rustls::Error::InvalidCertificate(rustls::CertificateError::UnknownIssuer)
            ))
            .to_string(),
            "certificate"
        );
    }
}
