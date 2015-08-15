mod inner;
mod pickup;
mod forward;
mod smtpd;

pub use self::inner::Process;
pub use self::inner::Inner;
pub use self::pickup::Pickup;
pub use self::forward::Forward;
pub use self::smtpd::Smtpd;

pub enum Message {
        Pickup { m: Pickup },
        Forward { m: Forward },
	Smtpd { m: Smtpd },
}
