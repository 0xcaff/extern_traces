use gcn::test_utils::GCNInstructionStream;
use snapshot_test_utils::generate_snapshots;

#[test]
fn snapshots() -> Result<(), anyhow::Error> {
    generate_snapshots(
        "test_data/captured/**/*.sb",
        "test_data/captured",
        "test_data/snapshots",
        |input, collector| {
            let shader = GCNInstructionStream::new(&input).unwrap();

            collector.result("debug", &shader.debug())?;

            collector.result("display", &shader.displayed())?;

            Ok(())
        },
    )?;

    Ok(())
}
