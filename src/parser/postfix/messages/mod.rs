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
pub use self::forward::Forward;
pub use self::smtpd::Smtpd;

pub enum Message {
	Cleanup { m: Cleanup },
        Pickup { m: Pickup },
	Qmgr { m: Qmgr},
        Forward { m: Forward },
	Smtpd { m: Smtpd },
}
