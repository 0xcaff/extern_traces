from regdb import load
from mako.template import Template

template = """
use strum::FromRepr;

#[repr(usize)]
#[allow(non_camel_case_types)]
#[derive(FromRepr, Debug)]
pub enum Register {
% for register_mapping in regdb.register_mappings():
    ${register_mapping.name} = ${register_mapping.map.at},
% endfor
}
"""

if __name__ == '__main__':
    regdb = load(['gfx7.json'])

    rendered = Template(template).render(
        regdb=regdb
    )

    with open('regs.rs', 'w') as file:
        file.write(rendered)
