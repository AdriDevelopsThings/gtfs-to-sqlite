BEGIN;
CREATE TABLE IF NOT EXISTS already_loaded (
    name TEXT UNIQUE
);
CREATE TABLE IF NOT EXISTS agency (
    agency_id INTEGER PRIMARY KEY,
    agency_name TEXT,
    agency_url TEXT,
    agency_timezone TEXT,
    agency_lang TEXT
);
CREATE TABLE IF NOT EXISTS attributions (
    attribution_id INTEGER PRIMARY KEY,
    organization_name TEXT,
    is_producer INTEGER,
    attribution_url TEXT,
    attribution_email TEXT
);
CREATE TABLE IF NOT EXISTS feed_info (
    feed_publisher_name TEXT,
    feed_publisher_url TEXT,
    feed_lang TEXT,
    feed_version TEXT,
    feed_contact_email TEXT,
    feed_contact_url TEXT
);
CREATE TABLE IF NOT EXISTS calendar (
    service_id INTEGER PRIMARY KEY,
    monday INTEGER,
    tuesday INTEGER,
    wednesday INTEGER,
    thursday INTEGER,
    friday INTEGER,
    saturday INTEGER,
    sunday INTEGER,
    start_date TEXT,
    end_date TEXT
);
CREATE TABLE IF NOT EXISTS calendar_dates (
    service_id INTEGER,
    exception_type INTEGER,
    date TEXT,
    FOREIGN KEY (service_id) REFERENCES calendar(service_id)
);
CREATE TABLE IF NOT EXISTS routes (
    route_id INTEGER PRIMARY KEY,
    route_long_name TEXT,
    route_short_name TEXT,
    route_type INTEGER,
    agency_id INTEGER,
    FOREIGN KEY (agency_id) REFERENCES agency(agency_id)
);
CREATE TABLE IF NOT EXISTS trips(
    trip_id INTEGER PRIMARY KEY,
    route_id INTEGER,
    service_id INTEGER,
    FOREIGN KEY (route_id) REFERENCES route(route_id),
    FOREIGN KEY (service_id) REFERENCES calendar(service_id)
);
CREATE TABLE IF NOT EXISTS stops (
    stop_id INTEGER PRIMARY KEY,
    stop_name TEXT,
    stop_lat REAL,
    stop_lon REAL,
    location_type INTEGER,
    parent_station INTEGER,
    FOREIGN KEY (parent_station) REFERENCES stops(stop_id) DEFERRABLE INITIALLY DEFERRED
);
CREATE TABLE IF NOT EXISTS stop_times (
    arrival_time TEXT,
    departure_time TEXT,
    stop_sequence INTEGER,
    pickup_type INTEGER,
    drop_off_type INTEGER,
    trip_id INTEGER,
    stop_id INTEGER,
    FOREIGN KEY (trip_id) REFERENCES trip(trip_id),
    FOREIGN KEY (stop_id) REFERENCES stop(stop_id)
);
COMMIT;