use c6_anonymization_identity::{Identity, LocalMac};

use crate::Error;

pub(crate) struct Plan {
    pub(crate) base: [u8; 6],
    pub(crate) identities: [Identity; 2],
}

impl Plan {
    pub(crate) fn from_sample(sample: [u8; 34]) -> Result<Self, Error> {
        // These guards reject known sentinels, not weak RNGs or replayed entropy.
        // Kconfig describes a 0x5a bring-up pattern; reject that sentinel too.
        // The bring-up bypass must independently be disabled at build time.
        if sample == [0; 34] || sample == [0x5a; 34] || sample[..6] == [0; 6] {
            return Err(Error::RandomSample);
        }
        let mut base_sample = [0; 6];
        base_sample.copy_from_slice(&sample[..6]);
        let base = LocalMac::from_random_bytes(base_sample).bytes();
        let mut first = [0; 14];
        first.copy_from_slice(&sample[6..20]);
        let mut second = [0; 14];
        second.copy_from_slice(&sample[20..]);
        let first = Identity::from_random_bytes(first).map_err(|_| Error::RandomSample)?;
        let second = Identity::from_random_bytes(second).map_err(|_| Error::RandomSample)?;
        if first.mac() == second.mac()
            || base == first.mac()
            || base == second.mac()
            || first.ssid() == second.ssid()
        {
            return Err(Error::IdentityCollision);
        }
        Ok(Self {
            base,
            identities: [first, second],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn distinct_sample() -> [u8; 34] {
        core::array::from_fn(|index| u8::try_from(index + 3).unwrap())
    }

    #[test]
    fn separates_domains_and_rejects_known_bypass_and_zero_samples() {
        assert!(matches!(
            Plan::from_sample([0; 34]),
            Err(Error::RandomSample)
        ));
        assert!(matches!(
            Plan::from_sample([0x5a; 34]),
            Err(Error::RandomSample)
        ));
        let plan = Plan::from_sample(distinct_sample()).unwrap();
        assert_ne!(plan.base, plan.identities[0].mac());
        assert_ne!(plan.identities[0].ssid(), plan.identities[1].ssid());
        assert_eq!(plan.base[0] & 3, 2);
    }

    #[test]
    fn rejects_collisions_after_mac_normalization_and_in_ssid_only() {
        let mut sample = distinct_sample();
        let first_mac: [u8; 6] = sample[6..12].try_into().unwrap();
        sample[20..26].copy_from_slice(&first_mac);
        sample[20] ^= 3;
        assert!(matches!(
            Plan::from_sample(sample),
            Err(Error::IdentityCollision)
        ));

        let mut sample = distinct_sample();
        let first_suffix: [u8; 8] = sample[12..20].try_into().unwrap();
        sample[26..34].copy_from_slice(&first_suffix);
        assert!(matches!(
            Plan::from_sample(sample),
            Err(Error::IdentityCollision)
        ));

        let mut sample = distinct_sample();
        sample[..6].copy_from_slice(&first_mac);
        assert!(matches!(
            Plan::from_sample(sample),
            Err(Error::IdentityCollision)
        ));
    }
}
