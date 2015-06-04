use std::fmt;

pub enum ParseError {
	DateTooShort,
	NonEndingHost,
	MissingProcess,
	NonEndingQueue,
	NonEndingProcess,
	BadProcessID,
	NonEndingQueueID,
	PickupBadUID,
	PickupBadFrom,
}

impl fmt::Display for ParseError {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let error = match self {
			&ParseError::DateTooShort => "Date Too Short",
			&ParseError::NonEndingHost => "Non Ending Host",
			&ParseError::MissingProcess => "Missing Process",
			&ParseError::NonEndingQueue => "Non Ending Queue",
			&ParseError::NonEndingProcess => "Non Ending Process",
			&ParseError::BadProcessID => "Bad Process ID",
			&ParseError::NonEndingQueueID => "Non Ending Queue ID",
			&ParseError::PickupBadUID => "Pickup Bad UID",
			&ParseError::PickupBadFrom => "Pickup Bad From",
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
		assert_print_eq(ParseError::BadProcessID, "Bad Process ID");
		assert_print_eq(ParseError::NonEndingQueueID, "Non Ending Queue ID");
		assert_print_eq(ParseError::PickupBadUID, "Pickup Bad UID");
		assert_print_eq(ParseError::PickupBadFrom, "Pickup Bad From");
	}
}
