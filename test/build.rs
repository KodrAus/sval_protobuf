use std::io::Result;

fn main() -> Result<()> {
    #[cfg(feature = "prost")]
    {
        let mut config = prost_build::Config::new();
        config.btree_map(&["."]);
        config.compile_protos(
            &[
                "protos/cases.proto",
                "protos/opentelemetry/proto/collector/logs/v1/logs_service.proto",
            ],
            &["protos/"],
        )?;
    }

    Ok(())
}
