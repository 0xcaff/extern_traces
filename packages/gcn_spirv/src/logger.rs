use rspirv::dr::{Builder, Operand};
use rspirv::spirv;
use std::iter;

pub struct Logger {
    void: spirv::Word,
    printf: spirv::Word,
    const_1: spirv::Word,
}

impl Logger {
    pub fn new(builder: &mut Builder, const_1: spirv::Word) -> Logger {
        let printf = builder.ext_inst_import("NonSemantic.DebugPrintf");
        builder.extension("SPV_KHR_non_semantic_info");

        Logger {
            printf,
            void: builder.type_void(),
            const_1,
        }
    }

    pub fn log(
        &self,
        builder: &mut Builder,
        fmt: &str,
        args: &[spirv::Word],
    ) -> Result<(), anyhow::Error> {
        let message = builder.string(fmt);
        let _ = builder.ext_inst(
            self.void,
            None,
            self.printf,
            self.const_1,
            iter::once(message)
                .chain(args.into_iter().map(|it| *it))
                .map(|it| Operand::IdRef(it)),
        )?;

        Ok(())
    }
}
