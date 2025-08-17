CREATE TABLE IF NOT EXISTS Picture (
    File_Path TEXT NOT NULL PRIMARY KEY,
    File_Size INTEGER,
    Colors INTEGER,
    Modified_Time INTEGER,
    Rank INTEGER,
    Palette BLOB,
    Label TEXT,
    Selected BOOLEAN,
    Deleted BOOLEAN);

CREATE TABLE IF NOT EXISTS Tag (
    File_Path TEXT NOT NULL,
    Label TEXT NOT NULL,
    PRIMARY KEY ( File_Path, Label));



