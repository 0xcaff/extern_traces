from aco_opcodes import opcodes
from mako.template import Template
from itertools import groupby

template = """
use strum::FromRepr;
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

impl ${format}OpCode {
    pub fn decode(op: usize) -> Result<Self, anyhow::Error> {
        Ok(Self::from_repr(op)
            .ok_or_else(|| anyhow::format_err!("unknown op {} for ${format}OpCode", op))?)
    }
}

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
    rendered = Template(template).render(ops=opcodes, to_option=to_option, to_sparse_array=to_sparse_array, groupby=groupby)
    with open('ops.rs', 'w') as file:
        file.write(rendered)
