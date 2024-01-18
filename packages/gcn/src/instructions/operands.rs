use bits::FromBits;
use std::fmt;

/// Also referred as SDST (**S**calar **D**e**s**ination) operand.
#[derive(Debug, Clone)]
pub enum ScalarDestinationOperand {
    /// Scalar **G**eneral **P**urpose **R**egister
    ScalarGPR(u8),

    VccLo,
    VccHi,

    M0,

    ExecLo,
    ExecHi,
}

impl FromBits<7> for ScalarDestinationOperand {
    fn from_bits(value: usize) -> Self {
        let value = value as u8;
        match value {
            0..=103 => ScalarDestinationOperand::ScalarGPR(value),
            106 => ScalarDestinationOperand::VccLo,
            107 => ScalarDestinationOperand::VccHi,
            124 => ScalarDestinationOperand::M0,
            126 => ScalarDestinationOperand::ExecLo,
            127 => ScalarDestinationOperand::ExecHi,
            _ => panic!("unknown {}", value),
        }
    }
}

impl fmt::Display for ScalarDestinationOperand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScalarDestinationOperand::ScalarGPR(idx) => write!(f, "s{}", *idx),
            ScalarDestinationOperand::VccLo => write!(f, "vcc_lo"),
            ScalarDestinationOperand::VccHi => write!(f, "vcc_hi"),
            ScalarDestinationOperand::M0 => write!(f, "m0"),
            ScalarDestinationOperand::ExecLo => write!(f, "exec_lo"),
            ScalarDestinationOperand::ExecHi => write!(f, "exec_hi"),
        }
    }
}

/// Also referred to as SSRC (**S**calar **S**ou**rc**e) operand.
#[derive(Debug, Clone)]
pub enum ScalarSourceOperand {
    Destination(ScalarDestinationOperand),
    IntegerConstant(InlineIntegerConstant),
    FloatConstant(InlineFloatConstant),

    VccZero,
    ExecZero,
    ScalarConditionCode,
    LDSDirect,
    LiteralConstant,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

impl FromBits<8> for ScalarSourceOperand {
    fn from_bits(value: usize) -> Self {
        let encoded = value as u8;
        match encoded {
            0..=127 => ScalarSourceOperand::Destination(ScalarDestinationOperand::from_bits(value)),

            128..=208 => {
                ScalarSourceOperand::IntegerConstant(InlineIntegerConstant { value: encoded })
            }
            240..=247 => ScalarSourceOperand::FloatConstant(InlineFloatConstant { value: encoded }),

            251 => ScalarSourceOperand::VccZero,
            252 => ScalarSourceOperand::ExecZero,
            253 => ScalarSourceOperand::ScalarConditionCode,
            254 => ScalarSourceOperand::LDSDirect,
            255 => ScalarSourceOperand::LiteralConstant,
            _ => panic!("unknown value {}", encoded),
        }
    }
}

impl fmt::Display for ScalarSourceOperand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScalarSourceOperand::Destination(dst) => write!(f, "{:?}", dst),
            ScalarSourceOperand::IntegerConstant(value) => write!(f, "{}", value.value()),
            ScalarSourceOperand::FloatConstant(value) => write!(f, "{}.f", value.value()),
            ScalarSourceOperand::VccZero => write!(f, "vccz"),
            ScalarSourceOperand::ExecZero => write!(f, "execz"),
            ScalarSourceOperand::ScalarConditionCode => write!(f, "scc"),
            ScalarSourceOperand::LDSDirect => {
                unimplemented!()
            }
            ScalarSourceOperand::LiteralConstant => {
                unimplemented!()
            }
        }
    }
}

#[derive(Debug)]
pub enum SourceOperand {
    Scalar(ScalarSourceOperand),
    VectorGPR(VectorGPR),
}

impl FromBits<9> for SourceOperand {
    fn from_bits(value: usize) -> Self {
        let value = value as u16;
        match value {
            0..=255 => SourceOperand::Scalar(ScalarSourceOperand::from_bits(value as usize)),
            256..=511 => SourceOperand::VectorGPR(VectorGPR {
                register_idx: (value - 256) as u8,
            }), // 256..=511 to VGPR0..VGPR255
            _ => panic!("unexpected value {}", value),
        }
    }
}

impl fmt::Display for SourceOperand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceOperand::Scalar(value) => write!(f, "{}", value),
            SourceOperand::VectorGPR(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Debug)]
pub struct VectorGPR {
    register_idx: u8,
}

impl FromBits<8> for VectorGPR {
    fn from_bits(value: usize) -> Self {
        let value = value as u8;
        VectorGPR {
            register_idx: value,
        }
    }
}

impl fmt::Display for VectorGPR {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}", self.register_idx)
    }
}

/// A group of 4 consecutive SGPR registers
#[derive(Debug)]
pub struct ScalarGeneralPurposeRegisterGroup {
    value: u8,
}

impl ScalarGeneralPurposeRegisterGroup {
    pub fn lowest_register(&self) -> ScalarDestinationOperand {
        ScalarDestinationOperand::from_bits((self.value << 2) as usize)
    }

    pub fn highest_register(&self) -> ScalarDestinationOperand {
        ScalarDestinationOperand::from_bits(((self.value << 2) + 3) as usize)
    }
}

impl FromBits<5> for ScalarGeneralPurposeRegisterGroup {
    fn from_bits(value: usize) -> Self {
        Self { value: value as _ }
    }
}

impl fmt::Display for ScalarGeneralPurposeRegisterGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "s[{}..{}]",
            match self.lowest_register() {
                ScalarDestinationOperand::ScalarGPR(gpr) => format!("{}", gpr),
                value => format!("{:?}", value),
            },
            match self.highest_register() {
                ScalarDestinationOperand::ScalarGPR(gpr) => format!("{}", gpr),
                value => format!("{:?}", value),
            },
        )
    }
}
