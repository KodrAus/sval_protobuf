pub mod prost;
pub mod sval;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_proto;

    use ::prost::Message;

    #[test]
    fn export_logs_service_request() {
        let prost = { prost::export_logs_service_request().encode_to_vec() };

        let sval1 = {
            sval_protobuf::stream_to_protobuf(sval::export_logs_service_request())
                .to_vec()
                .into_owned()
        };
        let sval2 = {
            let mut buf = Vec::new();
            sval_protobuf::stream_to_protobuf(sval::export_logs_service_request())
                .into_cursor()
                .copy_to_vec(&mut buf);
            buf
        };

        assert_proto(&prost, &sval1);
        assert_proto(&prost, &sval2);
    }

    #[bench]
    fn buffer(b: &mut test::Bencher) {
        b.iter(|| export_logs_service_request().encode_to_vec());
    }

    #[bench]
    fn buffer_sval(b: &mut test::Bencher) {
        b.iter(|| sval_protobuf::stream_to_protobuf(sval::export_logs_service_request()))
    }

    #[bench]
    fn buffer_sval_cursor(b: &mut test::Bencher) {
        b.iter(|| {
            let mut buf = Vec::new();
            sval_protobuf::stream_to_protobuf(sval::export_logs_service_request())
                .into_cursor()
                .copy_to_vec(&mut buf);
            buf
        })
    }

    #[bench]
    fn calculate_len(b: &mut test::Bencher) {
        let buf = sval_protobuf::stream_to_protobuf(sval::export_logs_service_request());

        b.iter(|| buf.len());
    }
}
