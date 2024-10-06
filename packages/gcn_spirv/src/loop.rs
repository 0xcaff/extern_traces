use rspirv::dr::Builder;
use rspirv::spirv;
use rspirv::spirv::LoopControl;

pub fn make_loop<State>(
    builder: &mut Builder,
    build_cmp: impl FnOnce(&mut Builder) -> Result<(spirv::Word, State), anyhow::Error>,
    build_body: impl FnOnce(&mut Builder, &State) -> Result<(), anyhow::Error>,
    build_increment: impl FnOnce(&mut Builder, &State) -> Result<(), anyhow::Error>,
) -> Result<(), anyhow::Error> {
    let loop_header = builder.id();
    builder.name(loop_header, "loop_header");

    builder.branch(loop_header)?;

    builder.begin_block(Some(loop_header))?;

    let merge_block = builder.id();
    builder.name(merge_block, "merge_block");
    let continue_target = builder.id();
    builder.name(continue_target, "continue_target");

    builder.loop_merge(merge_block, continue_target, LoopControl::NONE, [])?;

    let compare_block = builder.id();
    builder.branch(compare_block)?;

    builder.begin_block(Some(compare_block))?;

    let (cmp_result, state) = build_cmp(builder)?;

    let loop_body = builder.id();
    builder.branch_conditional(cmp_result, loop_body, merge_block, [])?;

    builder.begin_block(Some(loop_body))?;

    build_body(builder, &state)?;

    builder.branch(continue_target)?;

    builder.begin_block(Some(continue_target))?;

    build_increment(builder, &state)?;

    builder.branch(loop_header)?;

    builder.begin_block(Some(merge_block))?;

    Ok(())
}
