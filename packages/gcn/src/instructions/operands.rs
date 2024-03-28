use crate::instructions::instruction_info::OperandInfo;
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

impl ScalarDestinationOperand {
    pub fn display(&self, operand_info: &Option<OperandInfo>) -> String {
        match self {
            ScalarDestinationOperand::ScalarGPR(idx) => {
                let size = match operand_info {
                    Some(OperandInfo::Size(words)) => *words,
                    Some(_) => unimplemented!("not implemented"),
                    None => 1,
                };

                if size == 1 {
                    format!("s{}", *idx)
                } else {
                    format!("s[{}:{}]", *idx, *idx + size - 1)
                }
            }
            ScalarDestinationOperand::VccLo => "vcc_lo".to_string(),
            ScalarDestinationOperand::VccHi => "vcc_hi".to_string(),
            ScalarDestinationOperand::M0 => "m0".to_string(),
            ScalarDestinationOperand::ExecLo => "exec_lo".to_string(),
            ScalarDestinationOperand::ExecHi => "exec_hi".to_string(),
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

#[derive(Clone)]
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

impl fmt::Debug for InlineIntegerConstant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("InlineIntegerConstant")
            .field(&self.value())
            .finish()
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

impl ScalarSourceOperand {
    pub fn display(
        &self,
        operand_info: &Option<OperandInfo>,
        literal_constant: Option<u32>,
    ) -> String {
        match self {
            ScalarSourceOperand::Destination(dst) => dst.display(operand_info),
            ScalarSourceOperand::IntegerConstant(value) => format!("{}", value.value()),
            ScalarSourceOperand::FloatConstant(value) => format!("{}.f", value.value()),
            ScalarSourceOperand::VccZero => "vccz".to_string(),
            ScalarSourceOperand::ExecZero => "execz".to_string(),
            ScalarSourceOperand::ScalarConditionCode => "scc".to_string(),
            ScalarSourceOperand::LDSDirect => {
                unimplemented!()
            }
            ScalarSourceOperand::LiteralConstant => {
                format!("0x{:x}", literal_constant.unwrap())
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

impl SourceOperand {
    pub fn display(
        &self,
        operand_info: &Option<OperandInfo>,
        literal_constant: Option<u32>,
    ) -> String {
        match self {
            SourceOperand::Scalar(value) => value.display(operand_info, literal_constant),
            SourceOperand::VectorGPR(value) => value.display(operand_info),
        }
    }
}

#[derive(Debug)]
pub struct VectorGPR {
    pub register_idx: u8,
}

impl FromBits<8> for VectorGPR {
    fn from_bits(value: usize) -> Self {
        let value = value as u8;
        VectorGPR {
            register_idx: value,
        }
    }
}

impl VectorGPR {
    pub fn display(&self, operand_info: &Option<OperandInfo>) -> String {
        if let Some(OperandInfo::Exec) = operand_info {
            return "exec".to_string();
        }

        let size = match operand_info {
            Some(OperandInfo::Size(words)) => *words,
            Some(_) => unimplemented!("not implemented"),
            None => 1,
        };

        if size == 1 {
            format!("v{}", self.register_idx)
        } else {
            format!("v[{}:{}]", self.register_idx, self.register_idx + size - 1)
        }
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

impl ScalarGeneralPurposeRegisterGroup {
    pub fn display(&self) -> String {
        let operand_info = Some(OperandInfo::Size(4));
        self.lowest_register().display(&operand_info)
    }
}
