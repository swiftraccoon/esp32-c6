#[cfg(any(target_os = "espidf", test))]
use crate::Error;

/// One of the two identities prepared during exclusive early startup.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Epoch {
    /// The first start attempt; consumed even if preparation or Start fails.
    First,
    /// The second attempt, requiring an explicit checked Stop after First.
    Second,
}

#[cfg(target_os = "espidf")]
impl Epoch {
    pub(crate) const fn index(self) -> usize {
        match self {
            Self::First => 0,
            Self::Second => 1,
        }
    }
}

#[cfg(any(target_os = "espidf", test))]
#[derive(Default)]
pub(crate) struct Sequence {
    first_attempted: bool,
    second_attempted: bool,
    first_stopped: bool,
    pub(crate) current: Option<Epoch>,
}

#[cfg(any(target_os = "espidf", test))]
impl Sequence {
    pub(crate) fn begin(&mut self, epoch: Epoch) -> Result<(), Error> {
        match epoch {
            Epoch::First if !self.first_attempted => self.first_attempted = true,
            Epoch::Second if self.first_stopped && !self.second_attempted => {
                self.second_attempted = true;
            }
            _ => return Err(Error::Order),
        }
        self.current = Some(epoch);
        Ok(())
    }

    pub(crate) fn stopped(&mut self) {
        if self.current == Some(Epoch::First) {
            self.first_stopped = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stop_before_first_never_authorizes_second_or_retries() {
        let mut sequence = Sequence::default();
        sequence.stopped();
        assert_eq!(sequence.begin(Epoch::Second), Err(Error::Order));
        sequence.begin(Epoch::First).unwrap();
        assert_eq!(sequence.begin(Epoch::First), Err(Error::Order));
        assert_eq!(sequence.begin(Epoch::Second), Err(Error::Order));
        sequence.stopped();
        sequence.begin(Epoch::Second).unwrap();
        sequence.stopped();
        assert_eq!(sequence.begin(Epoch::First), Err(Error::Order));
        assert_eq!(sequence.begin(Epoch::Second), Err(Error::Order));
    }
}
