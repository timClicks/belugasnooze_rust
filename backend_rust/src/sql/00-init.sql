BEGIN;

CREATE TABLE IF NOT EXISTS alarm  (
	alarm_id INTEGER PRIMARY KEY AUTOINCREMENT,
	day INTEGER NOT NULL CHECK (day >= 0 AND day <= 6),
	hour INTEGER NOT NULL CHECK (hour >= 0 AND hour <= 23),
	minute INTEGER NOT NULL CHECK (minute >= 0 AND minute <= 59),
	UNIQUE (day, hour, minute)
) STRICT;
		
CREATE TABLE IF NOT EXISTS timezone  (
	timezone_id INTEGER PRIMARY KEY AUTOINCREMENT CHECK (timezone_id = 1),
	zone TEXT NOT NULL DEFAULT 'America/New_York',
	offset_hour INTEGER NOT NULL CHECK (offset_hour >= -23 AND offset_hour <= 23) DEFAULT -5,
	offset_minute INTEGER NOT NULL CHECK (offset_minute >= 0 AND offset_minute <= 59) DEFAULT 0,
	offset_second INTEGER NOT NULL CHECK (offset_second >= 0 AND offset_second <= 59) DEFAULT 0
) STRICT;

COMMIT;