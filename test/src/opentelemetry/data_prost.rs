use crate::protos::opentelemetry::proto::{
    collector::logs::v1::*, common::v1::*, logs::v1::*, resource::v1::*,
};

pub fn log_record1() -> LogRecord {
    LogRecord {
        time_unix_nano: 1696310935000000000,
        observed_time_unix_nano: 1696310935000000000,
        severity_number: SeverityNumber::Info as i32,
        severity_text: "Info".to_string(),
        body: Some(AnyValue { value: Some(any_value::Value::StringValue("Added 1 x product {\"Name\":\"Rocket Ship Dark Roast, Whole Beans\",\"SizeInGrams\":100} to order".to_string())) }),
        attributes: vec![
            KeyValue {
                key: "Action".to_string(),
                value: Some(AnyValue { value: Some(any_value::Value::StringValue("AddItem".to_string())) }),
            },
            KeyValue {
                key: "Controller".to_string(),
                value: Some(AnyValue { value: Some(any_value::Value::StringValue("OrdersController".to_string())) }),
            },
            KeyValue {
                key: "Application".to_string(),
                value: Some(AnyValue { value: Some(any_value::Value::StringValue("Roastery Web Frontend".to_string())) }),
            },
            KeyValue {
                key: "OrderId".to_string(),
                value: Some(AnyValue { value: Some(any_value::Value::StringValue("order-154613823ae87469e1edc0".to_string())) }),
            },
            KeyValue {
                key: "Origin".to_string(),
                value: Some(AnyValue { value: Some(any_value::Value::StringValue("seqcli sample ingest".to_string())) }),
            },
            KeyValue {
                key: "Product".to_string(),
                value: Some(AnyValue {
                    value: Some(any_value::Value::KvlistValue(KeyValueList {
                        values: vec![
                            KeyValue {
                                key: "Name".to_string(),
                                value: Some(AnyValue { value: Some(any_value::Value::StringValue("Rocket Ship Dark Roast, Whole Beans".to_string())) }),
                            },
                            KeyValue {
                                key: "SizeInGrams".to_string(),
                                value: Some(AnyValue { value: Some(any_value::Value::IntValue(100)) }),
                            },
                        ],
                    }))
                }),
            },
            KeyValue {
                key: "ProductId".to_string(),
                value: Some(AnyValue { value: Some(any_value::Value::StringValue("product-8908fd0sa".to_string())) }),
            },
            KeyValue {
                key: "RequestId".to_string(),
                value: Some(AnyValue { value: Some(any_value::Value::StringValue("b249369fe6c70fa27e4897".to_string())) }),
            },
            KeyValue {
                key: "SourceContext".to_string(),
                value: Some(AnyValue { value: Some(any_value::Value::StringValue("Roastery.Api.OrdersController".to_string())) }),
            },
        ],
        dropped_attributes_count: 0,
        flags: 0,
        trace_id: vec![],
        span_id: vec![],
    }
}

pub fn log_record2() -> LogRecord {
    LogRecord {
        time_unix_nano: 1696311466000000000,
        observed_time_unix_nano: 1696311466000000000,
        severity_number: SeverityNumber::Debug as i32,
        severity_text: "Debug".to_string(),
        body: Some(AnyValue { value: Some(any_value::Value::StringValue("Execution of insert into roastery.orderitem (orderid, productid) values ('order-724537b0f2348b2d60aabf', 'product-cvsad9033') returning id; affected 1 rows in 9.241 ms".to_string())) }),
        attributes: vec![
            KeyValue {
                key: "Action".to_string(),
                value: Some(AnyValue { value: Some(any_value::Value::StringValue("AddItem".to_string())) }),
            },
            KeyValue {
                key: "Controller".to_string(),
                value: Some(AnyValue { value: Some(any_value::Value::StringValue("OrdersController".to_string())) }),
            },
            KeyValue {
                key: "Application".to_string(),
                value: Some(AnyValue { value: Some(any_value::Value::StringValue("Roastery Web Frontend".to_string())) }),
            },
            KeyValue {
                key: "Elapsed".to_string(),
                value: Some(AnyValue { value: Some(any_value::Value::DoubleValue(9.241)) }),
            },
            KeyValue {
                key: "OrderId".to_string(),
                value: Some(AnyValue { value: Some(any_value::Value::StringValue("order-724537b0f2348b2d60aabf".to_string())) }),
            },
            KeyValue {
                key: "Origin".to_string(),
                value: Some(AnyValue { value: Some(any_value::Value::StringValue("seqcli sample ingest".to_string())) }),
            },
            KeyValue {
                key: "ProductId".to_string(),
                value: Some(AnyValue { value: Some(any_value::Value::StringValue("product-cvsad9033".to_string())) }),
            },
            KeyValue {
                key: "RequestId".to_string(),
                value: Some(AnyValue { value: Some(any_value::Value::StringValue("044ca890649f3111d0bddb".to_string())) }),
            },
            KeyValue {
                key: "RowCount".to_string(),
                value: Some(AnyValue { value: Some(any_value::Value::IntValue(1)) }),
            },
            KeyValue {
                key: "SourceContext".to_string(),
                value: Some(AnyValue { value: Some(any_value::Value::StringValue("Roastery.Data.Database".to_string())) }),
            },
            KeyValue {
                key: "Sql".to_string(),
                value: Some(AnyValue { value: Some(any_value::Value::StringValue("insert into roastery.orderitem (orderid, productid) values ('order-724537b0f2348b2d60aabf', 'product-cvsad9033') returning id;".to_string())) }),
            },
        ],
        dropped_attributes_count: 0,
        flags: 0,
        trace_id: vec![],
        span_id: vec![],
    }
}

pub fn log_record3() -> LogRecord {
    LogRecord {
        time_unix_nano: 1696311899000000000,
        observed_time_unix_nano: 1696311899000000000,
        severity_number: SeverityNumber::Error as i32,
        severity_text: "Error".to_string(),
        body: Some(AnyValue { value: Some(any_value::Value::StringValue("HTTP POST /api/orders/order-ad424d996f277c10e38056/items responded 500 in 10.211 ms".to_string())) }),
        attributes: vec![
            KeyValue {
                key: "Exception".to_string(),
                value: Some(AnyValue { value: Some(any_value::Value::StringValue("System.OperationCanceledException: A deadlock was detected and the transaction chosen as the deadlock victim.\n   at Roastery.Data.Database.LogExecAsync(String sql, Int32 rowCount) in /Users/user/dl/osx/seqcli/src/Roastery/Data/Database.cs:line 139\n   at Roastery.Data.Database.SelectAsync[T](Func`2 predicate, String where) in /Users/user/dl/osx/seqcli/src/Roastery/Data/Database.cs:line 55\n   at Roastery.Api.OrdersController.AddItem(HttpRequest request) in /Users/user/dl/osx/seqcli/src/Roastery/Api/OrdersController.cs:line 98\n   at Roastery.Web.Router.InvokeAsync(HttpRequest request) in /Users/user/dl/osx/seqcli/src/Roastery/Web/Router.cs:line 94\n   at Roastery.Web.FaultInjectionMiddleware.InvokeAsync(HttpRequest request) in /Users/user/dl/osx/seqcli/src/Roastery/Web/FaultInjectionMiddleware.cs:line 64\n   at Roastery.Web.SchedulingLatencyMiddleware.InvokeAsync(HttpRequest request) in /Users/user/dl/osx/seqcli/src/Roastery/Web/SchedulingLatencyMiddleware.cs:line 32\n   at Roastery.Web.RequestLoggingMiddleware.InvokeAsync(HttpRequest request) in /Users/user/dl/osx/seqcli/src/Roastery/Web/RequestLoggingMiddleware.cs:line 29".to_string())) }),
            },
            KeyValue {
                key: "Application".to_string(),
                value: Some(AnyValue { value: Some(any_value::Value::StringValue("Roastery Web Frontend".to_string())) }),
            },
            KeyValue {
                key: "Elapsed".to_string(),
                value: Some(AnyValue { value: Some(any_value::Value::DoubleValue(10.2111)) }),
            },
            KeyValue {
                key: "Origin".to_string(),
                value: Some(AnyValue { value: Some(any_value::Value::StringValue("seqcli sample ingest".to_string())) }),
            },
            KeyValue {
                key: "RequestId".to_string(),
                value: Some(AnyValue { value: Some(any_value::Value::StringValue("4e48b8a4a87cd9ecfb6e37".to_string())) }),
            },
            KeyValue {
                key: "RequestMethod".to_string(),
                value: Some(AnyValue { value: Some(any_value::Value::StringValue("POST".to_string())) }),
            },
            KeyValue {
                key: "RequestPath".to_string(),
                value: Some(AnyValue { value: Some(any_value::Value::StringValue("/api/orders/order-ad424d996f277c10e38056/items".to_string())) }),
            },
            KeyValue {
                key: "SourceContext".to_string(),
                value: Some(AnyValue { value: Some(any_value::Value::StringValue("Roastery.Web.RequestLoggingMiddleware".to_string())) }),
            },
            KeyValue {
                key: "StatusCode".to_string(),
                value: Some(AnyValue { value: Some(any_value::Value::IntValue(500)) }),
            },
        ],
        dropped_attributes_count: 0,
        flags: 0,
        trace_id: vec![],
        span_id: vec![],
    }
}

pub fn export_logs_service_request() -> ExportLogsServiceRequest {
    ExportLogsServiceRequest {
        resource_logs: vec![ResourceLogs {
            resource: Some(Resource {
                attributes: vec![KeyValue {
                    key: "service.name".to_string(),
                    value: Some(AnyValue {
                        value: Some(any_value::Value::StringValue(
                            "sval_protobuf_tests".to_owned(),
                        )),
                    }),
                }],
                dropped_attributes_count: 0,
            }),
            scope_logs: vec![ScopeLogs {
                scope: None,
                log_records: vec![log_record1(), log_record2(), log_record3()],
                schema_url: String::new(),
            }],
            schema_url: String::new(),
        }],
    }
}
