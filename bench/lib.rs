#![cfg(test)]
#![feature(test)]
extern crate test;

use ::prost::Message as _;

use sval_protobuf_test::opentelemetry::{data_prost, data_sval};

#[bench]
fn export_logs_service_request_prost(b: &mut test::Bencher) {
    b.iter(|| data_prost::export_logs_service_request().encode_to_vec());
}

#[bench]
fn export_logs_service_request_sval(b: &mut test::Bencher) {
    b.iter(|| sval_protobuf::stream_to_protobuf(data_sval::export_logs_service_request()))
}

#[bench]
fn export_logs_service_request_sval_pre_alloc(b: &mut test::Bencher) {
    use sval::Value as _;

    let mut stream = sval_protobuf::ProtoBufStream::new();
    data_sval::export_logs_service_request()
        .stream(&mut stream)
        .unwrap();
    let (_, reuse) = stream.freeze_reuse();

    let capacity = reuse.capacity();
    let mut reuse = Some(reuse.with_capacity(sval_protobuf::Capacity::next(&[capacity])));

    b.iter(|| {
        let mut stream = sval_protobuf::ProtoBufStream::new_reuse(reuse.take().unwrap());
        data_sval::export_logs_service_request()
            .stream(&mut stream)
            .unwrap();
        let next = stream.freeze_reuse();
        reuse = Some(next.1);
        next.0
    })
}

#[bench]
fn log_record1_prost(b: &mut test::Bencher) {
    b.iter(|| data_prost::log_record1().encode_to_vec());
}

#[bench]
fn log_record1_sval(b: &mut test::Bencher) {
    b.iter(|| sval_protobuf::stream_to_protobuf(data_sval::log_record1()))
}

#[bench]
fn log_record2_prost(b: &mut test::Bencher) {
    b.iter(|| data_prost::log_record2().encode_to_vec());
}

#[bench]
fn log_record2_sval(b: &mut test::Bencher) {
    b.iter(|| sval_protobuf::stream_to_protobuf(data_sval::log_record2()))
}

#[bench]
fn log_record3_prost(b: &mut test::Bencher) {
    b.iter(|| data_prost::log_record3().encode_to_vec());
}

#[bench]
fn log_record3_sval(b: &mut test::Bencher) {
    b.iter(|| sval_protobuf::stream_to_protobuf(data_sval::log_record3()))
}

#[bench]
fn export_logs_service_request_sval_cursor(b: &mut test::Bencher) {
    b.iter(|| {
        let mut buf = Vec::new();
        sval_protobuf::stream_to_protobuf(data_sval::export_logs_service_request())
            .into_cursor()
            .copy_to_vec(&mut buf);
        buf
    })
}

#[bench]
fn export_logs_service_request_len(b: &mut test::Bencher) {
    let buf = sval_protobuf::stream_to_protobuf(data_sval::export_logs_service_request());

    b.iter(|| buf.len());
}
