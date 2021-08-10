use prometheus::*;
use lazy_static::*;
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
    pub static ref SHARD_LATENCY_HISTOGRAM: HistogramVec =
        register_histogram_vec ! (
            "shard_operation_latency_ms",
            "Histogram of latency of executing an operation at shard level",
            & ["operation"],
            exponential_buckets(0.0001, 2.0, 20).unwrap()
        ).unwrap();
}


lazy_static! {
    pub static ref INIT_FILE_RECORDS_APPENDED: IntCounter =
        prometheus::register_int_counter!("init_file_records_appended", "Number of records appended during initial load").unwrap();
    pub static ref API_ADD_LATENCY: Histogram =
        prometheus::register_histogram!("init_file_records_appended", "Number of records appended during initial load").unwrap();
    pub static ref LOCAL_SHARD_LATENCY_HISTOGRAM: LocalShardLatencyHistogram = auto_flush_from!(SHARD_LATENCY_HISTOGRAM, LocalShardLatencyHistogram);
}

