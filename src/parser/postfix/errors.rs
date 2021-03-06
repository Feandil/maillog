use std::fmt;

pub enum ParseError {
	DateTooShort,
	NonEndingHost,
	MissingProcess,
	NonEndingQueue,
	NonEndingProcess,
	UnknownProcess,
	BadProcessID,
	BounceBad,
	BounceBadQueueID,
	PickupBadUID,
	PickupBadFrom,
	ForwardBadHost,
	ForwardNoMessage,
	ForwardNoTo,
	ForwardBadTo,
	ForwardBadOrigTo,
	ForwardNoRelay,
	ForwardBadRelay,
	ForwardBadConn,
	ForwardNoDelays,
	ForwardNoDelay,
	ForwardNoDSN,
	ForwardBadDSN,
	PickupDSNNotInt,
	ForwardDSNBadLen,
	ForwardNoStatus,
	SmtpdUnknownFormat,
	SmtpdNonEndingOrigQueue,
	SmtpdNoOrigClient,
	SmtpdNonEndingMethod,
	SmtpdUnknownMethod,
	SmtpdNoUsername,
	RejectBadMessage,
	RejectNoFrom,
	RejectBadFrom,
	RejectNoTo,
	RejectBadTo,
	RejectNoProto,
	RejectBadProto,
	RejectUnknownProto,
	RejectNoHelo,
	RejectBadHelo,
	CleanupNoMessageID,
	QmgrNoFrom,
	QmgrBadFrom,
	QmgrNoSize,
	QmgrBadSize,
	QmgrSizeNotInt,
	QmgrNoNrcpt,
	QmgrBadNrcpt,
	QmgrNotActive,
	QmgrNrcptNotInt,
}

impl fmt::Display for ParseError {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let error = match self {
			&ParseError::DateTooShort => "Date Too Short",
			&ParseError::NonEndingHost => "Non Ending Host",
			&ParseError::MissingProcess => "Missing Process",
			&ParseError::NonEndingQueue => "Non Ending Queue",
			&ParseError::NonEndingProcess => "Non Ending Process",
			&ParseError::UnknownProcess => "Unknown Process name",
			&ParseError::BadProcessID => "Bad Process ID",
			&ParseError::BounceBad => "Bounce unknown format",
			&ParseError::BounceBadQueueID => "Bounce child queue id illegal char",
			&ParseError::PickupBadUID => "Pickup Bad UID",
			&ParseError::PickupBadFrom => "Pickup Bad From",
			&ParseError::ForwardBadHost => "Forward non ending host",
			&ParseError::ForwardNoMessage => "Forward no message",
			&ParseError::ForwardNoTo => "Forward no To",
			&ParseError::ForwardBadTo => "Forward non ending To",
			&ParseError::ForwardBadOrigTo => "Forward non ending Orig_to",
			&ParseError::ForwardNoRelay => "Forward no Relay",
			&ParseError::ForwardBadRelay => "Forward non ending Relay",
			&ParseError::ForwardBadConn => "Forward non ending Conn_use",
			&ParseError::ForwardNoDelays => "Forward no Delays",
			&ParseError::ForwardNoDelay => "Forward no Delay",
			&ParseError::ForwardNoDSN => "Forward no DSN",
			&ParseError::ForwardBadDSN => "Forward non ending DSN",
			&ParseError::PickupDSNNotInt => "Forward DSN containing non u8",
			&ParseError::ForwardDSNBadLen => "Forward DSN not containing 3 u8",
			&ParseError::ForwardNoStatus => "Forward no Status",
			&ParseError::SmtpdUnknownFormat => "Smtpd with comma but no know format",
			&ParseError::SmtpdNonEndingMethod => "Smtpd with non ending method",
			&ParseError::SmtpdUnknownMethod => "Smtpd with unkown method",
			&ParseError::SmtpdNoUsername => "Smtpd without a username",
			&ParseError::SmtpdNonEndingOrigQueue => "Smtpd with origin queue ID but nothing else",
			&ParseError::SmtpdNoOrigClient => "Smtpd with origin queue ID but no origin client",
			&ParseError::RejectBadMessage => "Reject non ending message",
			&ParseError::RejectNoFrom => "Reject no from",
			&ParseError::RejectBadFrom => "Reject non ending from",
			&ParseError::RejectNoTo => "Reject no to",
			&ParseError::RejectBadTo => "Reject non ending to",
			&ParseError::RejectNoProto => "Reject no proto",
			&ParseError::RejectBadProto => "Reject non endin proto",
			&ParseError::RejectUnknownProto => "Reject unknown proto",
			&ParseError::RejectNoHelo => "Reject no helo",
			&ParseError::RejectBadHelo => "Reject non ending helo",
			&ParseError::CleanupNoMessageID => "Cleanup without any message id",
			&ParseError::QmgrNoFrom => "Qmgr no from",
			&ParseError::QmgrBadFrom => "Qmgr non ending from",
			&ParseError::QmgrNoSize => "Qmgr no size",
			&ParseError::QmgrBadSize => "Qmgr non ending size",
			&ParseError::QmgrSizeNotInt => "Qmgr size is not an int",
			&ParseError::QmgrNoNrcpt => "Qmgr no nrcpt",
			&ParseError::QmgrBadNrcpt => "Qmgr non ending nrcpt",
			&ParseError::QmgrNotActive => "Qmgr not in active queue",
			&ParseError::QmgrNrcptNotInt => "Qmgr nrcpt is not and int",
		};
		write!(fmt, "{}", error)
	}
}

impl fmt::Debug for ParseError {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(self, fmt)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::fmt;

	fn assert_print_eq(error: ParseError, expected: &'static str) {
		assert_eq!(fmt::format(format_args!("{}", error)), expected);
		assert_eq!(fmt::format(format_args!("{:?}", error)), expected);
	}

	#[test]
	fn formatting() {
		assert_print_eq(ParseError::DateTooShort, "Date Too Short");
		assert_print_eq(ParseError::NonEndingHost, "Non Ending Host");
		assert_print_eq(ParseError::MissingProcess, "Missing Process");
		assert_print_eq(ParseError::NonEndingQueue, "Non Ending Queue");
		assert_print_eq(ParseError::NonEndingProcess, "Non Ending Process");
		assert_print_eq(ParseError::UnknownProcess, "Unknown Process name");
		assert_print_eq(ParseError::BadProcessID, "Bad Process ID");
		assert_print_eq(ParseError::BounceBad, "Bounce unknown format");
		assert_print_eq(ParseError::BounceBadQueueID, "Bounce child queue id illegal char");
		assert_print_eq(ParseError::PickupBadUID, "Pickup Bad UID");
		assert_print_eq(ParseError::PickupBadFrom, "Pickup Bad From");
		assert_print_eq(ParseError::ForwardBadHost, "Forward non ending host");
		assert_print_eq(ParseError::ForwardNoMessage, "Forward no message");
		assert_print_eq(ParseError::ForwardNoTo, "Forward no To");
		assert_print_eq(ParseError::ForwardBadTo, "Forward non ending To");
		assert_print_eq(ParseError::ForwardBadOrigTo, "Forward non ending Orig_to");
		assert_print_eq(ParseError::ForwardNoRelay, "Forward no Relay");
		assert_print_eq(ParseError::ForwardBadRelay, "Forward non ending Relay");
		assert_print_eq(ParseError::ForwardBadConn, "Forward non ending Conn_use");
		assert_print_eq(ParseError::ForwardNoDelays, "Forward no Delays");
		assert_print_eq(ParseError::ForwardNoDelay, "Forward no Delay");
		assert_print_eq(ParseError::ForwardNoDSN, "Forward no DSN");
		assert_print_eq(ParseError::ForwardBadDSN, "Forward non ending DSN");
		assert_print_eq(ParseError::PickupDSNNotInt, "Forward DSN containing non u8");
		assert_print_eq(ParseError::ForwardDSNBadLen, "Forward DSN not containing 3 u8");
		assert_print_eq(ParseError::ForwardNoStatus, "Forward no Status");
		assert_print_eq(ParseError::SmtpdUnknownFormat, "Smtpd with comma but no know format");
		assert_print_eq(ParseError::SmtpdNonEndingMethod, "Smtpd with non ending method");
		assert_print_eq(ParseError::SmtpdUnknownMethod, "Smtpd with unkown method");
		assert_print_eq(ParseError::SmtpdNoUsername, "Smtpd without a username");
		assert_print_eq(ParseError::RejectBadMessage, "Reject non ending message");
		assert_print_eq(ParseError::RejectNoFrom, "Reject no from");
		assert_print_eq(ParseError::RejectBadFrom, "Reject non ending from");
		assert_print_eq(ParseError::RejectNoTo, "Reject no to");
		assert_print_eq(ParseError::RejectBadTo, "Reject non ending to");
		assert_print_eq(ParseError::RejectNoProto, "Reject no proto");
		assert_print_eq(ParseError::RejectBadProto, "Reject non endin proto");
		assert_print_eq(ParseError::RejectUnknownProto, "Reject unknown proto");
		assert_print_eq(ParseError::RejectNoHelo, "Reject no helo");
		assert_print_eq(ParseError::RejectBadHelo, "Reject non ending helo");
		assert_print_eq(ParseError::CleanupNoMessageID, "Cleanup without any message id");
		assert_print_eq(ParseError::QmgrNoFrom, "Qmgr no from");
		assert_print_eq(ParseError::QmgrBadFrom, "Qmgr non ending from");
		assert_print_eq(ParseError::QmgrNoSize, "Qmgr no size");
		assert_print_eq(ParseError::QmgrBadSize, "Qmgr non ending size");
		assert_print_eq(ParseError::QmgrSizeNotInt, "Qmgr size is not an int");
		assert_print_eq(ParseError::QmgrNoNrcpt, "Qmgr no nrcpt");
		assert_print_eq(ParseError::QmgrBadNrcpt, "Qmgr non ending nrcpt");
		assert_print_eq(ParseError::QmgrNotActive, "Qmgr not in active queue");
		assert_print_eq(ParseError::QmgrNrcptNotInt, "Qmgr nrcpt is not and int");
	}	
}
