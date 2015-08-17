mod inner;
mod cleanup;
mod pickup;
mod forward;
mod smtpd;

pub use self::inner::Process;
pub use self::inner::Inner;
pub use self::cleanup::Cleanup;
pub use self::pickup::Pickup;
pub use self::forward::Forward;
pub use self::smtpd::Smtpd;

pub enum Message {
	Cleanup { m: Cleanup },
        Pickup { m: Pickup },
        Forward { m: Forward },
	Smtpd { m: Smtpd },
}
