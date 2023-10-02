#[cfg(test)]
mod tests {
    use crate::*;

    use prost::Message;
    use sval_derive::*;
    use sval_protobuf::buf::ProtoBuf;

    #[derive(Value)]
    pub struct ExportLogsServiceRequest<'a> {
        #[sval(index = 1)]
        resource_logs: &'a [ResourceLogs<'a>],
    }

    #[derive(Value)]
    pub struct ResourceLogs<'a> {
        #[sval(index = 1)]
        resource: Option<Resource<'a>>,
        #[sval(index = 2)]
        scope_logs: &'a [ScopeLogs<'a>],
        #[sval(index = 3)]
        schema_url: &'a str,
    }

    #[derive(Value)]
    pub struct Resource<'a> {
        #[sval(index = 1)]
        attributes: &'a [KeyValue<'a>],
        #[sval(index = 2)]
        dropped_attribute_count: u32,
    }

    #[derive(Value)]
    pub struct ScopeLogs<'a> {
        #[sval(index = 1)]
        scope: Option<InstrumentationScope<'a>>,
        #[sval(index = 2)]
        log_records: &'a [LogRecord<'a>],
        #[sval(index = 3)]
        schema_url: &'a str,
    }

    #[derive(Value)]
    pub struct InstrumentationScope<'a> {
        #[sval(index = 1)]
        name: &'a str,
        #[sval(index = 2)]
        version: &'a str,
        #[sval(index = 3)]
        attributes: &'a [KeyValue<'a>],
        #[sval(index = 4)]
        dropped_attribute_count: u32,
    }

    #[derive(Value)]
    #[repr(i32)]
    pub enum SeverityNumber {
        Unspecified = 0,
        Trace = 1,
        Debug = 5,
        Info = 9,
        Warn = 13,
        Error = 17,
        Fatal = 21,
    }

    #[derive(Value)]
    pub enum AnyValue<'a> {
        #[sval(index = 1)]
        String(&'a str),
        #[sval(index = 2)]
        Bool(bool),
        #[sval(index = 3)]
        Int(i64),
        #[sval(index = 4)]
        Double(f64),
        #[sval(index = 5)]
        Array(&'a [AnyValue<'a>]),
        #[sval(index = 6)]
        Kvlist(KvList<'a>),
        #[sval(index = 7)]
        Bytes(&'a sval::BinarySlice),
    }

    #[derive(Value)]
    pub struct KvList<'a> {
        #[sval(index = 1)]
        values: &'a [KeyValue<'a>],
    }

    #[derive(Value)]
    pub struct KeyValue<'a> {
        #[sval(index = 1)]
        key: &'a str,
        #[sval(index = 2)]
        value: Option<AnyValue<'a>>,
    }

    #[derive(Value)]
    pub struct LogRecord<'a> {
        #[sval(index = 1, data_tag = "sval_protobuf::tags::PROTOBUF_I64")]
        time_unix_nano: u64,
        #[sval(index = 11, data_tag = "sval_protobuf::tags::PROTOBUF_I64")]
        observed_time_unix_nano: u64,
        #[sval(index = 2)]
        severity_number: SeverityNumber,
        #[sval(index = 3)]
        severity_text: &'a str,
        #[sval(index = 5)]
        body: Option<AnyValue<'a>>,
        #[sval(index = 6)]
        attributes: &'a [KeyValue<'a>],
        #[sval(index = 7)]
        dropped_attributes_count: u32,
        #[sval(index = 8, data_tag = "sval_protobuf::tags::PROTOBUF_I32")]
        flags: u32,
        #[sval(index = 9)]
        trace_id: &'a sval::BinaryArray<16>,
        #[sval(index = 10)]
        span_id: &'a sval::BinaryArray<8>,
    }

    fn export_logs_service_request_prost() -> Vec<u8> {
        let request =
            protos::opentelemetry::proto::collector::logs::v1::ExportLogsServiceRequest {
                resource_logs: vec![protos::opentelemetry::proto::logs::v1::ResourceLogs {
                    resource: Some(
                        protos::opentelemetry::proto::resource::v1::Resource {
                            attributes: vec![
                                protos::opentelemetry::proto::common::v1::KeyValue {
                                    key: "service.name".to_string(),
                                    value: Some(protos::opentelemetry::proto::common::v1::AnyValue {
                                        value: Some(protos::opentelemetry::proto::common::v1::any_value::Value::StringValue("smoke-test-rs".to_owned()))
                                    }),
                                },
                            ],
                            dropped_attributes_count: 0,
                        }
                    ),
                    scope_logs: vec![protos::opentelemetry::proto::logs::v1::ScopeLogs {
                        scope: None,
                        log_records: vec![
                            protos::opentelemetry::proto::logs::v1::LogRecord {
                                time_unix_nano: 0,
                                observed_time_unix_nano: 0,
                                severity_number: protos::opentelemetry::proto::logs::v1::SeverityNumber::Info as i32,
                                severity_text: "INFO".to_string(),
                                body: None,
                                attributes: vec![
                                    protos::opentelemetry::proto::common::v1::KeyValue {
                                        key: "a".to_string(),
                                        value: Some(protos::opentelemetry::proto::common::v1::AnyValue {
                                            value: Some(protos::opentelemetry::proto::common::v1::any_value::Value::KvlistValue(protos::opentelemetry::proto::common::v1::KeyValueList {
                                                values: vec![
                                                    protos::opentelemetry::proto::common::v1::KeyValue {
                                                        key: "a1".to_string(),
                                                        value: Some(protos::opentelemetry::proto::common::v1::AnyValue {
                                                            value: Some(protos::opentelemetry::proto::common::v1::any_value::Value::BoolValue(true)),
                                                        }),
                                                    },
                                                    protos::opentelemetry::proto::common::v1::KeyValue {
                                                        key: "a2".to_string(),
                                                        value: Some(protos::opentelemetry::proto::common::v1::AnyValue {
                                                            value: Some(protos::opentelemetry::proto::common::v1::any_value::Value::IntValue(4)),
                                                        }),
                                                    }
                                                ]
                                            }))
                                        }),
                                    },
                                    protos::opentelemetry::proto::common::v1::KeyValue {
                                        key: "b".to_string(),
                                        value: Some(protos::opentelemetry::proto::common::v1::AnyValue {
                                            value: Some(protos::opentelemetry::proto::common::v1::any_value::Value::StringValue("text".to_string())),
                                        }),
                                    }
                                ],
                                dropped_attributes_count: 0,
                                flags: 0,
                                trace_id: vec![0; 16],
                                span_id: vec![0; 8],
                            },
                        ],
                        schema_url: String::new(),
                    }],
                    schema_url: String::new(),
                }],
            };

        request.encode_to_vec()
    }

    fn export_logs_service_request_sval() -> ProtoBuf {
        let request = ExportLogsServiceRequest {
            resource_logs: &[ResourceLogs {
                resource: Some(Resource {
                    attributes: &[KeyValue {
                        key: "service.name",
                        value: Some(AnyValue::String("smoke-test-rs")),
                    }],
                    dropped_attribute_count: 0,
                }),
                scope_logs: &[ScopeLogs {
                    scope: None,
                    log_records: &[LogRecord {
                        time_unix_nano: 0,
                        observed_time_unix_nano: 0,
                        severity_number: SeverityNumber::Info,
                        severity_text: "INFO",
                        body: None,
                        attributes: &[
                            KeyValue {
                                key: "a",
                                value: Some(AnyValue::Kvlist(KvList {
                                    values: &[
                                        KeyValue {
                                            key: "a1",
                                            value: Some(AnyValue::Bool(true)),
                                        },
                                        KeyValue {
                                            key: "a2",
                                            value: Some(AnyValue::Int(4)),
                                        },
                                    ],
                                })),
                            },
                            KeyValue {
                                key: "b",
                                value: Some(AnyValue::String("text")),
                            },
                        ],
                        dropped_attributes_count: 0,
                        flags: 0,
                        trace_id: sval::BinaryArray::new(&[0; 16]),
                        span_id: sval::BinaryArray::new(&[0; 8]),
                    }],
                    schema_url: "",
                }],
                schema_url: "",
            }],
        };

        sval_protobuf::stream_to_protobuf(&request)
    }

    #[test]
    fn export_logs_service_request() {
        let prost = { export_logs_service_request_prost() };

        let sval1 = { export_logs_service_request_sval().to_vec().into_owned() };
        let sval2 = {
            let mut buf = Vec::new();
            export_logs_service_request_sval()
                .into_cursor()
                .copy_to_vec(&mut buf);
            buf
        };

        assert_proto(&prost, &sval1);
        assert_proto(&prost, &sval2);
    }

    #[bench]
    fn buffer_prost(b: &mut test::Bencher) {
        b.iter(|| export_logs_service_request_prost());
    }

    #[bench]
    fn buffer_sval(b: &mut test::Bencher) {
        b.iter(|| export_logs_service_request_sval())
    }

    #[bench]
    fn buffer_sval_cursor(b: &mut test::Bencher) {
        b.iter(|| {
            let mut buf = Vec::new();
            export_logs_service_request_sval()
                .into_cursor()
                .copy_to_vec(&mut buf);
            buf
        })
    }

    #[bench]
    fn calculate_len(b: &mut test::Bencher) {
        let buf = export_logs_service_request_sval();

        b.iter(|| buf.len());
    }
}
