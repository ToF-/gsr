CREATE TABLE IF NOT EXISTS Picture (
    File_Path TEXT PRIMARY KEY,
    File_Size INTEGER,
    Colors INTEGER,
    Modified_Time INTEGER,
    Rank INTEGER,
    Palette BLOB,
    Label TEXT,
    Selected BOOLEAN,
    Deleted BOOLEAN);



