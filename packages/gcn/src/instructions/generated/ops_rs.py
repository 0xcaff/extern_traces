from aco_opcodes import opcodes
from mako.template import Template

template = """
pub enum OpFormat {
    SOP1,
    SOP2,
    SOPK,
    SOPP,
    SOPC,
    // Scalar Memory Format
    SMEM,
    // LDS/GDS Format
    DS,
    LDSDIR,
    // Vector Memory Buffer Formats
    MTBUF,
    MUBUF,
    // Vector Memory Image Format
    MIMG,
    // Export Format
    EXP,
    // Flat Formats
    FLAT,
    GLOBAL,
    SCRATCH,
    // Vector Parameter Interpolation Formats
    VINTRP,
    // Vector ALU Formats
    VINTERP_INREG,
    VOP1,
    VOP2,
    VOPC,
    VOP3,
    VOP3P,
    SDWA,
    DPP16,
    DPP8,
}

pub enum OpClass {
    Valu32,
    ValuConvert32,
    Valu64,
    ValuQuarterRate32,
    ValuFma,
    ValuTranscendental32,
    ValuDouble,
    ValuDoubleAdd,
    ValuDoubleConvert,
    ValuDoubleTranscendental,
    WMMA,
    Salu,
    SMem,
    Barrier,
    Branch,
    Sendmsg,
    DS,
    Export,
    VMem,
    Waitcnt,
    Other,
}

pub struct OpInfo {
    pub op_code: OpCode,
    pub name: &'static str,
    pub format: OpFormat,
    pub class: OpClass,

    pub gfx7: Option<u16>,
    pub gfx9: Option<u16>,
    pub gfx10: Option<u16>,
    pub gfx11: Option<u16>,
}

#[allow(non_camel_case_types)]
pub enum OpCode {
   % for idx, op in enumerate(ops.values()):
   ${op.name} = ${idx},
   % endfor
}

<%def name="render_gfx_level_ops(ops, prefix, extract)">
<% gfx_level_ops = to_sparse_array(ops.values(), extract) %>
pub const ${prefix}_OPS: [Option<OpCode>; ${len(gfx_level_ops)}] = [
    % for op in gfx_level_ops:
    % if op:
    Some(OpCode::${op.name}),
    % else:
    None,
    % endif
    % endfor
];
</%def>

${render_gfx_level_ops(ops, "GFX7", lambda it: it.opcode_gfx7)}
${render_gfx_level_ops(ops, "GFX9", lambda it: it.opcode_gfx9)}
${render_gfx_level_ops(ops, "GFX10", lambda it: it.opcode_gfx10)}
${render_gfx_level_ops(ops, "GFX11", lambda it: it.opcode_gfx11)}

pub const OPS: [OpInfo; ${len(ops)}] = [
    % for op in ops.values():
    OpInfo {
        op_code: OpCode::${op.name},
        name: "${op.name}",
        format: OpFormat::${op.format.name},
        class: OpClass::${op.cls.name},
        gfx7: ${to_option(op.opcode_gfx7)},
        gfx9: ${to_option(op.opcode_gfx9)},
        gfx10: ${to_option(op.opcode_gfx10)},
        gfx11: ${to_option(op.opcode_gfx11)},
    },
    % endfor
];
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
    rendered = Template(template).render(ops=opcodes, to_option=to_option, to_sparse_array=to_sparse_array)
    with open('ops.rs', 'w') as file:
        file.write(rendered)
