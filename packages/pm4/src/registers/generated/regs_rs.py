import math
from itertools import filterfalse

from regdb import load
from mako.template import Template

overrides = {
    "SPI_SHADER_FORMAT": 4,
    "VGT_TESS_TOPOLOGY": 3,
    "VGT_GS_OUTPRIM_TYPE": 6,
    "CP_PERFMON_STATE": 4,
    "SPM_PERFMON_STATE": 4,
    "PA_SU_SC_MODE_CNTL__POLY_MODE": 2,
    "PA_SU_SC_MODE_CNTL__POLYMODE_FRONT_PTYPE": 3,
    "VGT_INDEX_TYPE_MODE": 2,
    "VGT_DI_MAJOR_MODE_SELECT": 2,
    "VGT_DI_PRIM_TYPE": 6,
    "VGT_TESS_PARTITION": 3,
}

# todo: remove these
ignored = [
    "EXCP_EN",
    "VS_EN",
    "GS_EN",
    "ES_EN",
]

template = """
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use strum::FromRepr;
use bits_macros::FromBits;
use bits::FromBits;
use crate::registers::usize::Usize;

#[repr(usize)]
#[derive(FromRepr, Debug)]
pub enum Register {
% for register_mapping in regdb.register_mappings():
    ${register_mapping.name} = ${register_mapping.map.at},
% endfor
}

% for (name, entries) in regdb.enums():
#[repr(usize)]
#[derive(FromRepr, Debug)]
pub enum ${name} {
% for entry in entries.entries:
    ${entry.name} = ${entry.value},
% endfor
}

impl FromBits<${ceil(log2(1 + max(map(lambda it: it.value, entries.entries))))}> for ${name} {
    fn from_bits(value: usize) -> Self {
        Self::from_repr(value).unwrap()
    }
}

% if name in overrides:
impl FromBits<${overrides[name]}> for ${name} {
    fn from_bits(value: usize) -> Self {
        Self::from_repr(value).unwrap()
    }
}

% endif

% endfor

% for (name, fields) in regdb.register_types():
#[derive(FromBits)]
#[bits(32)]
pub struct ${name} {
% for field in unique(fields.fields, lambda it: it.name):
    % if field.name not in ignored:
    #[bits(${field.bits[1]}, ${field.bits[0]})]
    ${field.name}: ${field.enum_ref if hasattr(field, 'enum_ref') else "Usize"},
    % endif
% endfor
}

% endfor
"""


def unique_everseen(iterable, key=None):
    "List unique elements, preserving order. Remember all elements ever seen."
    # unique_everseen('AAAABBBCCDAABBB') --> A B C D
    # unique_everseen('ABBcCAD', str.casefold) --> A B c D
    seen = set()
    if key is None:
        for element in filterfalse(seen.__contains__, iterable):
            seen.add(element)
            yield element
    else:
        for element in iterable:
            k = key(element)
            if k not in seen:
                seen.add(k)
                yield element


if __name__ == '__main__':
    regdb = load(['gfx7.json'])

    rendered = Template(template).render(
        regdb=regdb,
        unique=unique_everseen,
        ceil=math.ceil,
        log2=math.log2,
        ignored=ignored,
        overrides=overrides,
    )

    with open('regs.rs', 'w') as file:
        file.write(rendered)
