# gtfs-to-sqlite
A rust program that reads gtfs data from a zip archive and inserts the data in a sqlite database.

## Installation 
Clone this repository with
```
git clone https://github.com/adridevelopsthings/gtfs-to-sqlite.git
```

and compile the code into a binary with
```
cargo build --release
```

## Usage
Prepare your gtfs data zip file that contains at least
- agency.txt
- attributions.txt
- feed_info.txt
- calendar.txt
- calendar_dates.txt
- routes.txt
- trips.txt
- stops.txt
- stop_times.txt

and set the environment variable `GTFS_FILE` to the path to this zip file (using a dotenv file is also possible). You could also set the environment variable `SQLITE_PATH` to the path where your sqlite database should be created but you could also use the default value `./database.sqlite`.

Now run
```
cargo run --release
```
and wait.

Take a look in the database schema [here](src/tables.sql).