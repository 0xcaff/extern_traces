/// Also referred as SDST (**S**calar **D**e**s**ination) operand.
pub enum ScalarDestinationOperand {
    /// Scalar **G**eneral **P**urpose **R**egister
    ScalarGPR(u8),

    VccLo,
    VccHi,

    M0,

    ExecLo,
    ExecHi,

    Reserved(u8),
}

impl ScalarDestinationOperand {
    pub fn decode(encoded: u8) -> ScalarDestinationOperand {
        let value = encoded & 0b1111111;
        debug_assert_eq!(value, encoded);

        match value {
            0..=103 => ScalarDestinationOperand::ScalarGPR(value),
            106 => ScalarDestinationOperand::VccLo,
            107 => ScalarDestinationOperand::VccHi,
            124 => ScalarDestinationOperand::M0,
            126 => ScalarDestinationOperand::ExecLo,
            127 => ScalarDestinationOperand::ExecHi,
            _ => ScalarDestinationOperand::Reserved(value),
        }
    }
}

/// Also referred to as SSRC (**S**calar **S**ou**rc**e) operand.
pub enum ScalarSourceOperand {
    Destination(ScalarDestinationOperand),
    IntegerConstant(InlineIntegerConstant),
    FloatConstant(InlineFloatConstant),

    VccZero,
    ExecZero,
    ScalarConditionCode,
    LDSDirect,
    LiteralConstant,

    Reserved(u8),
}

pub struct InlineIntegerConstant {
    value: u8,
}

impl InlineIntegerConstant {
    pub fn value(&self) -> i8 {
        match self.value {
            128 => 0,
            129..=192 => (self.value as i16 - 129 + 1) as i8, // maps 129..=192 to 1..64
            193..=208 => (-(self.value as i16 - 193 + 1)) as i8, // maps 193..208 to -1..-16
            _ => panic!("unexpected value {}", self.value),
        }
    }
}

pub struct InlineFloatConstant {
    value: u8,
}

impl InlineFloatConstant {
    pub fn value(&self) -> f32 {
        match self.value {
            240 => 0.5,
            241 => -0.5,
            242 => 1.0,
            243 => -1.0,
            244 => 2.0,
            245 => -2.0,
            246 => 4.0,
            247 => -4.0,
            _ => panic!("unexpected value {}", self.value),
        }
    }
}

impl ScalarSourceOperand {
    pub fn decode(encoded: u8) -> ScalarSourceOperand {
        match encoded {
            0..=127 => ScalarSourceOperand::Destination(ScalarDestinationOperand::decode(encoded)),

            128..=208 => {
                ScalarSourceOperand::IntegerConstant(InlineIntegerConstant { value: encoded })
            }
            240..=247 => ScalarSourceOperand::FloatConstant(InlineFloatConstant { value: encoded }),

            251 => ScalarSourceOperand::VccZero,
            252 => ScalarSourceOperand::ExecZero,
            253 => ScalarSourceOperand::ScalarConditionCode,
            254 => ScalarSourceOperand::LDSDirect,
            255 => ScalarSourceOperand::LiteralConstant,
            _ => ScalarSourceOperand::Reserved(encoded),
        }
    }
}

pub enum SourceOperand {
    Scalar(ScalarSourceOperand),
    VectorGPR(VectorGPR),
}

impl SourceOperand {
    pub fn decode(value: u16) -> SourceOperand {
        match value {
            0..=255 => SourceOperand::Scalar(ScalarSourceOperand::decode(value as u8)),
            256..=511 => SourceOperand::VectorGPR(VectorGPR {
                register_idx: (value - 256) as u8,
            }), // 256..=511 to VGPR0..VGPR255
            _ => panic!("unexpected value {}", value),
        }
    }
}

pub struct VectorGPR {
    register_idx: u8,
}

impl VectorGPR {
    pub fn decode(value: u8) -> VectorGPR {
        VectorGPR {
            register_idx: value,
        }
    }
}
