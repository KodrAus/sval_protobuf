#[cfg(feature = "prost")]
pub mod data_prost;

pub mod data_sval;

#[cfg(all(test, feature = "prost"))]
mod tests {
    use super::*;

    use ::prost::Message;

    #[test]
    fn export_logs_service_request() {
        let prost = data_prost::export_logs_service_request();

        let sval1 = {
            sval_protobuf::stream_to_protobuf(data_sval::export_logs_service_request())
                .to_vec()
                .into_owned()
        };
        let sval2 = {
            let mut buf = Vec::new();
            sval_protobuf::stream_to_protobuf(data_sval::export_logs_service_request())
                .into_cursor()
                .copy_to_vec(&mut buf);
            buf
        };

        let decoded_prost1 = crate::protos::opentelemetry::proto::collector::logs::v1::ExportLogsServiceRequest::decode(std::io::Cursor::new(&sval1)).expect("failed to decode");
        let decoded_prost2 = crate::protos::opentelemetry::proto::collector::logs::v1::ExportLogsServiceRequest::decode(std::io::Cursor::new(&sval2)).expect("failed to decode");

        assert_eq!(prost, decoded_prost1);
        assert_eq!(prost, decoded_prost2);
    }
}
