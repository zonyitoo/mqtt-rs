//! QoS (Quality of Services)

use crate::packet::publish::QoSWithPacketIdentifier;

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub enum QualityOfService {
    Level0 = 0,
    Level1 = 1,
    Level2 = 2,
}

impl From<QoSWithPacketIdentifier> for QualityOfService {
    fn from(qos: QoSWithPacketIdentifier) -> Self {
        match qos {
            QoSWithPacketIdentifier::Level0 => QualityOfService::Level0,
            QoSWithPacketIdentifier::Level1(_) => QualityOfService::Level1,
            QoSWithPacketIdentifier::Level2(_) => QualityOfService::Level2,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::cmp::min;

    #[test]
    fn min_qos() {
        let q1 = QoSWithPacketIdentifier::Level1(0).into();
        let q2 = QualityOfService::Level2;
        assert_eq!(min(q1, q2), q1);

        let q1 = QoSWithPacketIdentifier::Level0.into();
        let q2 = QualityOfService::Level2;
        assert_eq!(min(q1, q2), q1);

        let q1 = QoSWithPacketIdentifier::Level2(0).into();
        let q2 = QualityOfService::Level1;
        assert_eq!(min(q1, q2), q2);
    }
}
