from aco_opcodes import opcodes
from mako.template import Template
from itertools import groupby

FORMATS = {
    "SOPC": 7,
    "DS": 8,
    "MIMG": 7,
    "MTBUF": 3,
    "MUBUF": 7,
    "SMEM": 5,
    "SOP1": 8,
    "SOP2": 7,
    "SOPK": 5,
    "SOPP": 7,
    "VINTRP": 2,
    "VOP1": 8,
    "VOP2": 6,
    "VOP3": 9,
    "VOPC": 8,
}

template = """
use strum::FromRepr;
use bits::FromBits;
use anyhow;

% for format, format_ops in groupby(filter(lambda op: op.opcode_gfx7 >= 0, ops.values()), lambda op: op.format.name):
#[repr(usize)]
#[allow(non_camel_case_types)]
#[derive(Eq, PartialEq, Clone, Copy, Debug, FromRepr)]
pub enum ${format}OpCode {
    % for op in sorted(format_ops, key=lambda op: op.opcode_gfx7):
    ${op.name} = ${op.opcode_gfx7},
    % endfor
}

% if format in FORMATS:
impl FromBits<${FORMATS[format]}> for ${format}OpCode {
    fn from_bits(value: usize) -> Self {
        Self::from_repr(value).unwrap()
    }
}
%endif
% endfor
"""


def to_sparse_array(ops, fn):
    relevant_ops = [item for item in ops if fn(item) != -1]
    max_index = max(fn(item) for item in relevant_ops)

    result = [None] * (max_index + 1)

    for item in relevant_ops:
        result[fn(item)] = item

    return result


def to_option(number):
    return f"Some({hex(number)})" if number >= 0 else "None"


if __name__ == '__main__':
    rendered = Template(template).render(
        ops=opcodes,
        to_option=to_option,
        to_sparse_array=to_sparse_array,
        groupby=groupby,
        FORMATS=FORMATS
    )
    with open('ops.rs', 'w') as file:
        file.write(rendered)
