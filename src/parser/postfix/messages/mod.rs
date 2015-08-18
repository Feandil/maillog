mod inner;
mod cleanup;
mod pickup;
mod forward;
mod qmgr;
mod smtpd;

pub use self::inner::Process;
pub use self::inner::Inner;
pub use self::cleanup::Cleanup;
pub use self::pickup::Pickup;
pub use self::qmgr::Qmgr;
pub use self::qmgr::QmgrRemoved;
pub use self::qmgr::QmgrExpired;
pub use self::forward::Forward;
pub use self::smtpd::Smtpd;
pub use self::smtpd::SmtpdForward;
pub use self::smtpd::SmtpdLogin;

use super::ParseError;

#[derive(Debug)]
pub enum Message {
	Cleanup { m: Cleanup },
        Pickup { m: Pickup },
	Qmgr { m: Qmgr},
	QmgrRemoved { m: QmgrRemoved },
	QmgrExpired { m: QmgrExpired },
        Forward { m: Forward },
	Smtpd { m: Smtpd },
	SmtpdForward { m: SmtpdForward },
	SmtpdLogin { m: SmtpdLogin },
}

pub trait MessageParser {
	fn parse(inner: Inner, start: usize) -> Result<Option<Message>, ParseError>;
}
