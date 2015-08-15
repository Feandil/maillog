use std::fmt;

pub enum ParseError {
	DateTooShort,
	NonEndingHost,
	MissingProcess,
	NonEndingQueue,
	NonEndingProcess,
	UnknownProcess,
	BadProcessID,
	PickupBadUID,
	PickupBadFrom,
	ForwardNoTo,
	ForwardBadTo,
	ForwardBadOrigTo,
	ForwardNoRelay,
	ForwardBadRelay,
	ForwardNoDelays,
	ForwardNoDelay,
	ForwardNoDSN,
	ForwardBadDSN,
	PickupDSNNotInt,
	ForwardDSNBadLen,
	ForwardNoStatus
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
			&ParseError::PickupBadUID => "Pickup Bad UID",
			&ParseError::PickupBadFrom => "Pickup Bad From",
			&ParseError::ForwardNoTo => "Forward no To",
			&ParseError::ForwardBadTo => "Forward non ending To",
			&ParseError::ForwardBadOrigTo => "Forward non ending Orig_to",
			&ParseError::ForwardNoRelay => "Forward no Relay",
			&ParseError::ForwardBadRelay => "Forward non ending Relay",
			&ParseError::ForwardNoDelays => "Forward no Delays",
			&ParseError::ForwardNoDelay => "Forward no Delay",
			&ParseError::ForwardNoDSN => "Forward no DSN",
			&ParseError::ForwardBadDSN => "Forward non ending DSN",
			&ParseError::PickupDSNNotInt => "Forward DSN containing non u8",
			&ParseError::ForwardDSNBadLen => "Forward DSN not containing 3 u8",
			&ParseError::ForwardNoStatus => "Forward no Status"
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
		assert_print_eq(ParseError::PickupBadUID, "Pickup Bad UID");
		assert_print_eq(ParseError::PickupBadFrom, "Pickup Bad From");
		assert_print_eq(ParseError::ForwardNoTo, "Forward no To");
		assert_print_eq(ParseError::ForwardBadTo, "Forward non ending To");
		assert_print_eq(ParseError::ForwardBadOrigTo, "Forward non ending Orig_to");
		assert_print_eq(ParseError::ForwardNoRelay, "Forward no Relay");
		assert_print_eq(ParseError::ForwardBadRelay, "Forward non ending Relay");
		assert_print_eq(ParseError::ForwardNoDelays, "Forward no Delays");
		assert_print_eq(ParseError::ForwardNoDelay, "Forward no Delay");
		assert_print_eq(ParseError::ForwardNoDSN, "Forward no DSN");
		assert_print_eq(ParseError::ForwardBadDSN, "Forward non ending DSN");
		assert_print_eq(ParseError::PickupDSNNotInt, "Forward DSN containing non u8");
		assert_print_eq(ParseError::ForwardDSNBadLen, "Forward DSN not containing 3 u8");
		assert_print_eq(ParseError::ForwardNoStatus, "Forward no Status");
	}
}
