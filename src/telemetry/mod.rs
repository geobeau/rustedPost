use lazy_static::*;
use prometheus::*;
use prometheus_static_metric::{auto_flush_from, make_auto_flush_static_metric};

make_auto_flush_static_metric! {
    pub label_enum Operations {
        add,
        raw_add,
        search,
        key_values_search,
    }

    pub struct LocalShardLatencyHistogram: LocalHistogram {
        "operation" => Operations,
    }
}

lazy_static! {
    pub static ref SHARD_LATENCY_HISTOGRAM: HistogramVec = register_histogram_vec!(
        "shard_operation_latency",
        "Histogram of latency of executing an operation at shard level",
        &["operation"],
        vec![
            0.000_001, 0.000_0025, 0.000_005, 0.000_01, 0.000_025, 0.000_05, 0.000_1, 0.000_25, 0.000_5, 0.001, 0.0025, 0.005, 0.010, 0.050, 0.1,
            0.25, 0.5, 1.0, 2.5, 5.0
        ]
    )
    .unwrap();
}

lazy_static! {
    pub static ref INIT_FILE_RECORDS_APPENDED: IntCounter =
        prometheus::register_int_counter!("init_file_records_appended", "Number of records appended during initial load").unwrap();
    pub static ref API_ADD_LATENCY: Histogram =
        prometheus::register_histogram!("init_file_records_appended", "Number of records appended during initial load").unwrap();
    pub static ref LOCAL_SHARD_LATENCY_HISTOGRAM: LocalShardLatencyHistogram = auto_flush_from!(SHARD_LATENCY_HISTOGRAM, LocalShardLatencyHistogram);
}
