use anyhow::Result;

use crate::{database::Database, gtfs::GtfsFile};

mod csv_reader;
mod database;
mod gtfs;

fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let mut database = Database::by_env()?;
    database.conn.execute_batch(
        "PRAGMA journal_mode = OFF; PRAGMA synchronous = OFF; PRAGMA temp_store = MEMORY;",
    )?;
    let mut gtfs = GtfsFile::by_env()?;
    gtfs.import(&mut database, "agency.txt", "agency")?;
    gtfs.import(&mut database, "attributions.txt", "attributions")?;
    gtfs.import(&mut database, "feed_info.txt", "feed_info")?;
    gtfs.import(&mut database, "calendar.txt", "calendar")?;
    gtfs.import(&mut database, "calendar_dates.txt", "calendar_dates")?;
    gtfs.import(&mut database, "routes.txt", "routes")?;
    gtfs.import(&mut database, "trips.txt", "trips")?;
    gtfs.import(&mut database, "stops.txt", "stops")?;
    gtfs.import(&mut database, "stop_times.txt", "stop_times")?;
    Ok(())
}
