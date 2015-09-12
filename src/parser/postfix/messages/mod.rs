mod inner;
mod bounce;
mod cleanup;
mod pickup;
mod forward;
mod qmgr;
mod reject;
mod smtpd;

pub use self::inner::Process;
pub use self::inner::Inner;
pub use self::bounce::Bounce;
pub use self::cleanup::Cleanup;
pub use self::pickup::Pickup;
pub use self::qmgr::Qmgr;
pub use self::qmgr::QmgrRemoved;
pub use self::qmgr::QmgrExpired;
pub use self::forward::Forward;
pub use self::forward::ForwardError;
pub use self::reject::Reject;
pub use self::reject::RejectReason;
pub use self::reject::RejectProto;
pub use self::smtpd::Smtpd;
pub use self::smtpd::SmtpdForward;
pub use self::smtpd::SmtpdLogin;

use super::ParseError;

#[derive(Debug)]
pub enum Message {
	Bounce { m: Bounce },
	Cleanup { m: Cleanup },
        Pickup { m: Pickup },
	Qmgr { m: Qmgr},
	QmgrRemoved { m: QmgrRemoved },
	QmgrExpired { m: QmgrExpired },
        Forward { m: Forward },
        ForwardError { m: ForwardError },
	Reject { m: Reject },
	Smtpd { m: Smtpd },
	SmtpdForward { m: SmtpdForward },
	SmtpdLogin { m: SmtpdLogin },
}

pub trait MessageParser {
	fn parse(inner: Inner, start: usize) -> Result<Option<Message>, ParseError>;
}
