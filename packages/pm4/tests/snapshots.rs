use pm4::{convert, PM4Packet};
use snapshot_test_utils::generate_snapshots;

#[test]
fn snapshots() -> Result<(), anyhow::Error> {
    generate_snapshots(
        "test_data/captures/**/*.pm4",
        "test_data/captures",
        "test_data/snapshots",
        |bytes, collector| {
            let packets = PM4Packet::parse_all(bytes.as_slice())?;

            collector.result("packets", &format!("{:#?}", packets))?;

            collector.result("pipeline", &format!("{:#?}", convert(packets.as_slice())))?;

            Ok(())
        },
    )
}
