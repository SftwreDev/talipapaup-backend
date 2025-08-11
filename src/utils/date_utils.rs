use chrono::{FixedOffset, Offset, TimeZone, Utc};
use chrono_tz::Asia::Manila;
use sea_orm::prelude::DateTimeWithTimeZone;

pub fn local_datetime() -> DateTimeWithTimeZone {
    let manila_time = Utc::now().with_timezone(&Manila);
    let offset_seconds = manila_time.offset().fix().local_minus_utc();
    let manila_offset = FixedOffset::east_opt(offset_seconds).unwrap();
    let now: DateTimeWithTimeZone = manila_offset.from_utc_datetime(&manila_time.naive_local()).into();

    now
}