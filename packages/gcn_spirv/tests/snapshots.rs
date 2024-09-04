use gcn::test_utils::GCNInstructionStream;
use gcn_spirv::module::construct_control_flow_graph;
use snapshot_test_utils::generate_snapshots;

#[test]
fn snapshots() -> Result<(), anyhow::Error> {
    generate_snapshots(
        "../gcn/test_data/captured/**/*.sb",
        "../gcn/test_data/captured",
        "../gcn/test_data/snapshots",
        |input, collector| {
            let shader = GCNInstructionStream::new(&input).unwrap();

            let graph = construct_control_flow_graph(&shader.instructions);

            collector.result("graph", &format!("{:#?}", graph))?;

            Ok(())
        },
    )?;

    Ok(())
}
